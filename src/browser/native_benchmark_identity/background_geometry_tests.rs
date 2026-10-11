use super::*;

#[test]
fn identity_default_bytes_stay_stable_and_each_repeat_axis_changes_identity() {
    let base = super::super::gradient_tests::command();
    let mut bytes = Bytes::new().unwrap();
    commands(&mut bytes, std::slice::from_ref(&base)).unwrap();
    let old = bytes.0;
    assert_eq!(&old[..5], &[1, 0, 0, 0, 10]);
    let mut expected = vec![1, 0, 0, 0, 10];
    for value in [-1.0f32, 2.0, 20.0, 12.0, 1.0, 3.0, 16.0, 8.0, 2.0] {
        expected.extend_from_slice(&value.to_le_bytes());
    }
    expected.extend_from_slice(&[12, 34, 56, 78]);
    expected.extend_from_slice(&1.25f32.to_le_bytes());
    expected.extend_from_slice(&[1, 1, 0]); // corner right/top
    expected.extend_from_slice(&2u32.to_le_bytes());
    expected.extend_from_slice(&[1, 1]); // currentColor and explicit position
    expected.extend_from_slice(&2.5f64.to_le_bytes());
    expected.extend_from_slice(&25.0f64.to_le_bytes());
    expected.extend_from_slice(&[0, 0, 0, 0, 255, 0]); // black, omitted position
    assert_eq!(old, expected);
    for (repeat, mask) in [([false, false], 0), ([true, false], 1), ([false, true], 2)] {
        let mut value = base.clone();
        let DrawCommand::LinearGradient { repeat: flags, .. } = &mut value else {
            panic!()
        };
        *flags = repeat;
        let mut bytes = Bytes::new().unwrap();
        commands(&mut bytes, &[value]).unwrap();
        let mut expected = old.clone();
        expected[4] = 11;
        expected.insert(5, mask);
        assert_eq!(bytes.0, expected);
    }
}
