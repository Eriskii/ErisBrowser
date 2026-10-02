//! Paid canonical payloads over private, repeatable UTF-16 streams.
use super::*;
use crate::dom::DomString;

// Only borrowed slices and DomUnits/take/chain/skip streams enter this helper.
// Keep the original iterator in the plan: no callback or replacement source
// may intervene between classification, fresh DOM admission and emission.
pub(super) struct DomDataPlan<I> {
    input: I,
    units: usize,
    bytes: usize,
    source_work: usize,
    pass_work: usize,
    nonscalar: bool,
}
impl<I> DomDataPlan<I> {
    pub(super) fn stored_bytes(&self) -> usize {
        self.bytes
    }
}
fn overflow() -> ScriptError {
    ScriptError::resource("DOM data work or storage overflow")
}

impl Runtime {
    pub(super) fn plan_dom_data<I>(
        &mut self,
        input: I,
        result_units: usize,
        source_work: usize,
    ) -> Result<DomDataPlan<I>>
    where
        I: Iterator<Item = u16> + Clone,
    {
        let pass_work = result_units
            .checked_mul(2)
            .and_then(|units| source_work.checked_add(units))
            .and_then(|work| work.checked_add(8))
            .ok_or_else(overflow)?;
        self.work(pass_work)?;
        let mut units = 0usize;
        let mut bytes = 0usize;
        let mut nonscalar = false;
        for value in char::decode_utf16(input.clone()) {
            let consumed = match value {
                Ok(value) => {
                    bytes = bytes.checked_add(value.len_utf8()).ok_or_else(overflow)?;
                    value.len_utf16()
                }
                Err(_) => {
                    nonscalar = true;
                    1
                }
            };
            units = units.checked_add(consumed).ok_or_else(overflow)?;
        }
        if units != result_units {
            return Err(ScriptError::resource("inconsistent DOM data stream length"));
        }
        if nonscalar {
            bytes = units
                .checked_mul(std::mem::size_of::<u16>())
                .ok_or_else(overflow)?;
        }
        Ok(DomDataPlan {
            input,
            units,
            bytes,
            source_work,
            pass_work,
            nonscalar,
        })
    }

    pub(super) fn emit_dom_data<I>(&mut self, plan: DomDataPlan<I>) -> Result<DomString>
    where
        I: Iterator<Item = u16> + Clone,
    {
        if plan.nonscalar {
            let work = plan
                .source_work
                .checked_add(plan.units)
                .and_then(|work| work.checked_add(8))
                .ok_or_else(overflow)?;
            self.work(work)?;
            self.charge(plan.bytes)?;
            let mut output = Vec::new();
            output
                .try_reserve_exact(plan.units)
                .map_err(|_| ScriptError::resource("DOM data units allocation failed"))?;
            output.extend(plan.input);
            // This is a separate reached validation, not the consumed plan scan.
            self.work(
                plan.units
                    .checked_mul(2)
                    .and_then(|n| n.checked_add(1))
                    .ok_or_else(overflow)?,
            )?;
            DomString::from_nonscalar_units(output)
                .map_err(|_| ScriptError::resource("inconsistent DOM data canonical form"))
        } else {
            self.work(
                plan.pass_work
                    .checked_add(plan.bytes)
                    .ok_or_else(overflow)?,
            )?;
            self.charge(plan.bytes)?;
            let mut output = String::new();
            output
                .try_reserve_exact(plan.bytes)
                .map_err(|_| ScriptError::resource("DOM data scalar allocation failed"))?;
            for value in char::decode_utf16(plan.input) {
                output.push(
                    value
                        .map_err(|_| ScriptError::resource("inconsistent DOM data scalar form"))?,
                );
            }
            Ok(output.into())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn build(runtime: &mut Runtime, units: &[u16]) -> Result<DomString> {
        let plan = runtime.plan_dom_data(units.iter().copied(), units.len(), units.len())?;
        runtime.emit_dom_data(plan)
    }

    #[test]
    fn canonical_builder_uses_one_payload_and_preserves_complete_units() {
        for (units, scalar, bytes) in [
            (vec![], true, 0),
            (vec![0x41, 0xd834, 0xdd1e, 0x20ac], true, 8),
            (vec![0xd800, 0x41, 0xd834, 0xdd1e, 0xdc00], false, 10),
        ] {
            let mut runtime = Runtime::new();
            let before = runtime.allocated;
            let value = build(&mut runtime, &units).unwrap();
            assert_eq!(value.units().collect::<Vec<_>>(), units);
            assert_eq!(value.scalar().is_some(), scalar);
            assert_eq!(value.stored_bytes(), bytes);
            assert_eq!(runtime.allocated - before, bytes);
        }
    }

    #[test]
    fn canonical_builder_measured_work_and_storage_boundaries_are_terminal() {
        for units in [vec![0x41, 0xd834, 0xdd1e], vec![0xd800, 0x41, 0xdc00]] {
            let mut witness = Runtime::new();
            let before = (witness.steps, witness.allocated);
            build(&mut witness, &units).unwrap();
            let (work, heap) = (before.0 - witness.steps, witness.allocated - before.1);
            for (steps, available, succeeds) in [
                (work, heap, true),
                (work - 1, heap, false),
                (work, heap - 1, false),
            ] {
                let mut runtime = Runtime::new();
                runtime.steps = steps;
                runtime.allocated = MAX_HEAP - available;
                let result = build(&mut runtime, &units);
                if succeeds {
                    assert_eq!(result.unwrap().units().collect::<Vec<_>>(), units);
                    assert_eq!(runtime.steps, 0);
                    assert_eq!(runtime.allocated, MAX_HEAP);
                } else {
                    assert!(result.unwrap_err().is_resource_limit());
                }
            }
        }
    }

    #[test]
    fn canonical_builder_classifies_after_first_isolated_unit_and_across_boundaries() {
        let old = DomString::from_units_owned(vec![0xd800, 0x78, 0xdc00]).unwrap();
        let mut runtime = Runtime::new();
        let input = old.units().take(1).chain(old.units().skip(2));
        let plan = runtime
            .plan_dom_data(input, 2, 2 * old.stored_bytes())
            .unwrap();
        assert_eq!(plan.stored_bytes(), 4);
        let value = runtime.emit_dom_data(plan).unwrap();
        assert_eq!(value.scalar(), Some("\u{10000}"));
        let input = [0xd800, 0x41, 0x42, 0xdc00];
        assert_eq!(
            build(&mut runtime, &input).unwrap().raw_units(),
            Some(input.as_slice())
        );
    }
}

#[cfg(test)]
mod independent_tests {
    use super::*;
    const CASES: &str = include_str!("../../tests/fixtures/dom-production.js");
    fn independent_case(name: &str) {
        for strict in [false, true] {
            let mut runtime = Runtime::try_new().unwrap();
            let mut doc = Document::parse("<p>kept</p>");
            let source = format!("{CASES}\ndomProductionCases.{name}();");
            let result = if strict {
                runtime.execute_strict(&source, &mut doc)
            } else {
                runtime.execute(&source, &mut doc)
            };
            assert_eq!(result.unwrap(), Value::Bool(true), "{name} strict={strict}");
            assert!(runtime.frames.is_empty());
            assert_eq!(
                (runtime.calls, runtime.eval_depth, runtime.stack_units),
                (0, 0, 0)
            );
        }
    }
    macro_rules! independent {($($name:ident),* $(,)?) => {$(#[test]fn $name(){independent_case(stringify!($name));})*};}
    independent!(
        constructors_and_factory_exact,
        defaults_and_required_arguments,
        setter_defaults_and_exact,
        authentic_brands_precede_conversion,
        units_substring_and_bounds,
        complete_splice_repairs,
        splice_retains_other_isolated_units,
        conversions_use_fresh_data,
        constructor_conversion_before_prototype,
        constructor_abrupt_preserves_callback_prefix,
        data_json_and_clone_roundtrips,
        pi_validation_creation_versus_mutation,
    );
}
