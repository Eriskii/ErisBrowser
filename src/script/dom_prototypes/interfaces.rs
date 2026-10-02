//! Static initial interface identity for the represented DOM node kinds.
//!
//! Mapping sources (read 2026-10-02):
//! - https://dom.spec.whatwg.org/#interface-document
//! - https://html.spec.whatwg.org/multipage/indices.html#elements-3
//! - https://html.spec.whatwg.org/multipage/dom.html#elements-in-the-dom
//! - https://html.spec.whatwg.org/multipage/obsolete.html
//! - https://svgwg.org/svg2-draft/idl.html
//! - https://svgwg.org/svg2-draft/types.html#SVGDOMElements
//! - https://svgwg.org/specs/animations/
//! - https://www.w3.org/TR/filter-effects-1/#DOMInterfaces
//! - https://drafts.csswg.org/css-masking/#dom-interfaces
//! - https://w3c.github.io/mathml-core/#dom-and-javascript
//!
//! These names identify initial prototypes, not complete interface APIs. No
//! custom-element upgrade, arbitrary namespace, Attr, CDATASection, XMLDocument
//! or ShadowRoot is represented here. HTMLDocument is an alias for Document;
//! it must not create a second interface/prototype pair. A template fragment's
//! host does not turn the fragment into a ShadowRoot. Mutable prototype
//! overrides and authentic receiver brands belong to the caller.

use crate::dom::{Document, Namespace, NodeId, NodeKind};
use std::cmp::Ordering;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ConstructorKind {
    /// No constructor operation: ordinary Call and attempted construction throw
    /// TypeError. WebIDL issue #698 leaves [[Construct]] presence ambiguous.
    Illegal,
    EventTarget,
    Document,
    DocumentFragment,
    Text,
    Comment,
    /// The current DOM standard declares a real (target, optional data) ctor.
    ProcessingInstruction,
    /// [HTMLConstructor]: direct new throws; successful construction requires
    /// custom-element registry/construction-stack semantics, outside this map.
    Html,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct Interface {
    pub(super) name: &'static str,
    /// None means Object.prototype for instances, Function.prototype for the
    /// interface object. Otherwise both chains use the named parent interface.
    pub(super) parent: Option<&'static str>,
    pub(super) constructor: ConstructorKind,
    pub(super) length: usize,
}

const fn entry(
    name: &'static str,
    parent: Option<&'static str>,
    constructor: ConstructorKind,
    length: usize,
) -> Interface {
    Interface {
        name,
        parent,
        constructor,
        length,
    }
}

/// Parent-before-child order. EventTarget already exists and must be reused.
pub(super) const INTERFACES: &[Interface] = &[
    entry("EventTarget", None, ConstructorKind::EventTarget, 0),
    entry("Node", Some("EventTarget"), ConstructorKind::Illegal, 0),
    entry("CharacterData", Some("Node"), ConstructorKind::Illegal, 0),
    entry(
        "Comment",
        Some("CharacterData"),
        ConstructorKind::Comment,
        0,
    ),
    entry("Document", Some("Node"), ConstructorKind::Document, 0),
    entry(
        "DocumentFragment",
        Some("Node"),
        ConstructorKind::DocumentFragment,
        0,
    ),
    entry("DocumentType", Some("Node"), ConstructorKind::Illegal, 0),
    entry("Element", Some("Node"), ConstructorKind::Illegal, 0),
    entry("HTMLElement", Some("Element"), ConstructorKind::Html, 0),
    entry(
        "HTMLAnchorElement",
        Some("HTMLElement"),
        ConstructorKind::Html,
        0,
    ),
    entry(
        "HTMLAreaElement",
        Some("HTMLElement"),
        ConstructorKind::Html,
        0,
    ),
    entry(
        "HTMLMediaElement",
        Some("HTMLElement"),
        ConstructorKind::Illegal,
        0,
    ),
    entry(
        "HTMLAudioElement",
        Some("HTMLMediaElement"),
        ConstructorKind::Html,
        0,
    ),
    entry(
        "HTMLBRElement",
        Some("HTMLElement"),
        ConstructorKind::Html,
        0,
    ),
    entry(
        "HTMLBaseElement",
        Some("HTMLElement"),
        ConstructorKind::Html,
        0,
    ),
    entry(
        "HTMLBodyElement",
        Some("HTMLElement"),
        ConstructorKind::Html,
        0,
    ),
    entry(
        "HTMLButtonElement",
        Some("HTMLElement"),
        ConstructorKind::Html,
        0,
    ),
    entry(
        "HTMLCanvasElement",
        Some("HTMLElement"),
        ConstructorKind::Html,
        0,
    ),
    entry(
        "HTMLDListElement",
        Some("HTMLElement"),
        ConstructorKind::Html,
        0,
    ),
    entry(
        "HTMLDataElement",
        Some("HTMLElement"),
        ConstructorKind::Html,
        0,
    ),
    entry(
        "HTMLDataListElement",
        Some("HTMLElement"),
        ConstructorKind::Html,
        0,
    ),
    entry(
        "HTMLDetailsElement",
        Some("HTMLElement"),
        ConstructorKind::Html,
        0,
    ),
    entry(
        "HTMLDialogElement",
        Some("HTMLElement"),
        ConstructorKind::Html,
        0,
    ),
    entry(
        "HTMLDirectoryElement",
        Some("HTMLElement"),
        ConstructorKind::Html,
        0,
    ),
    entry(
        "HTMLDivElement",
        Some("HTMLElement"),
        ConstructorKind::Html,
        0,
    ),
    entry(
        "HTMLEmbedElement",
        Some("HTMLElement"),
        ConstructorKind::Html,
        0,
    ),
    entry(
        "HTMLFieldSetElement",
        Some("HTMLElement"),
        ConstructorKind::Html,
        0,
    ),
    entry(
        "HTMLFontElement",
        Some("HTMLElement"),
        ConstructorKind::Html,
        0,
    ),
    entry(
        "HTMLFormElement",
        Some("HTMLElement"),
        ConstructorKind::Html,
        0,
    ),
    entry(
        "HTMLFrameElement",
        Some("HTMLElement"),
        ConstructorKind::Html,
        0,
    ),
    entry(
        "HTMLFrameSetElement",
        Some("HTMLElement"),
        ConstructorKind::Html,
        0,
    ),
    entry(
        "HTMLHRElement",
        Some("HTMLElement"),
        ConstructorKind::Html,
        0,
    ),
    entry(
        "HTMLHeadElement",
        Some("HTMLElement"),
        ConstructorKind::Html,
        0,
    ),
    entry(
        "HTMLHeadingElement",
        Some("HTMLElement"),
        ConstructorKind::Html,
        0,
    ),
    entry(
        "HTMLHtmlElement",
        Some("HTMLElement"),
        ConstructorKind::Html,
        0,
    ),
    entry(
        "HTMLIFrameElement",
        Some("HTMLElement"),
        ConstructorKind::Html,
        0,
    ),
    entry(
        "HTMLImageElement",
        Some("HTMLElement"),
        ConstructorKind::Html,
        0,
    ),
    entry(
        "HTMLInputElement",
        Some("HTMLElement"),
        ConstructorKind::Html,
        0,
    ),
    entry(
        "HTMLLIElement",
        Some("HTMLElement"),
        ConstructorKind::Html,
        0,
    ),
    entry(
        "HTMLLabelElement",
        Some("HTMLElement"),
        ConstructorKind::Html,
        0,
    ),
    entry(
        "HTMLLegendElement",
        Some("HTMLElement"),
        ConstructorKind::Html,
        0,
    ),
    entry(
        "HTMLLinkElement",
        Some("HTMLElement"),
        ConstructorKind::Html,
        0,
    ),
    entry(
        "HTMLMapElement",
        Some("HTMLElement"),
        ConstructorKind::Html,
        0,
    ),
    entry(
        "HTMLMarqueeElement",
        Some("HTMLElement"),
        ConstructorKind::Html,
        0,
    ),
    entry(
        "HTMLMenuElement",
        Some("HTMLElement"),
        ConstructorKind::Html,
        0,
    ),
    entry(
        "HTMLMetaElement",
        Some("HTMLElement"),
        ConstructorKind::Html,
        0,
    ),
    entry(
        "HTMLMeterElement",
        Some("HTMLElement"),
        ConstructorKind::Html,
        0,
    ),
    entry(
        "HTMLModElement",
        Some("HTMLElement"),
        ConstructorKind::Html,
        0,
    ),
    entry(
        "HTMLOListElement",
        Some("HTMLElement"),
        ConstructorKind::Html,
        0,
    ),
    entry(
        "HTMLObjectElement",
        Some("HTMLElement"),
        ConstructorKind::Html,
        0,
    ),
    entry(
        "HTMLOptGroupElement",
        Some("HTMLElement"),
        ConstructorKind::Html,
        0,
    ),
    entry(
        "HTMLOptionElement",
        Some("HTMLElement"),
        ConstructorKind::Html,
        0,
    ),
    entry(
        "HTMLOutputElement",
        Some("HTMLElement"),
        ConstructorKind::Html,
        0,
    ),
    entry(
        "HTMLParagraphElement",
        Some("HTMLElement"),
        ConstructorKind::Html,
        0,
    ),
    entry(
        "HTMLParamElement",
        Some("HTMLElement"),
        ConstructorKind::Html,
        0,
    ),
    entry(
        "HTMLPictureElement",
        Some("HTMLElement"),
        ConstructorKind::Html,
        0,
    ),
    entry(
        "HTMLPreElement",
        Some("HTMLElement"),
        ConstructorKind::Html,
        0,
    ),
    entry(
        "HTMLProgressElement",
        Some("HTMLElement"),
        ConstructorKind::Html,
        0,
    ),
    entry(
        "HTMLQuoteElement",
        Some("HTMLElement"),
        ConstructorKind::Html,
        0,
    ),
    entry(
        "HTMLScriptElement",
        Some("HTMLElement"),
        ConstructorKind::Html,
        0,
    ),
    entry(
        "HTMLSelectElement",
        Some("HTMLElement"),
        ConstructorKind::Html,
        0,
    ),
    entry(
        "HTMLSelectedContentElement",
        Some("HTMLElement"),
        ConstructorKind::Html,
        0,
    ),
    entry(
        "HTMLSlotElement",
        Some("HTMLElement"),
        ConstructorKind::Html,
        0,
    ),
    entry(
        "HTMLSourceElement",
        Some("HTMLElement"),
        ConstructorKind::Html,
        0,
    ),
    entry(
        "HTMLSpanElement",
        Some("HTMLElement"),
        ConstructorKind::Html,
        0,
    ),
    entry(
        "HTMLStyleElement",
        Some("HTMLElement"),
        ConstructorKind::Html,
        0,
    ),
    entry(
        "HTMLTableCaptionElement",
        Some("HTMLElement"),
        ConstructorKind::Html,
        0,
    ),
    entry(
        "HTMLTableCellElement",
        Some("HTMLElement"),
        ConstructorKind::Html,
        0,
    ),
    entry(
        "HTMLTableColElement",
        Some("HTMLElement"),
        ConstructorKind::Html,
        0,
    ),
    entry(
        "HTMLTableElement",
        Some("HTMLElement"),
        ConstructorKind::Html,
        0,
    ),
    entry(
        "HTMLTableRowElement",
        Some("HTMLElement"),
        ConstructorKind::Html,
        0,
    ),
    entry(
        "HTMLTableSectionElement",
        Some("HTMLElement"),
        ConstructorKind::Html,
        0,
    ),
    entry(
        "HTMLTemplateElement",
        Some("HTMLElement"),
        ConstructorKind::Html,
        0,
    ),
    entry(
        "HTMLTextAreaElement",
        Some("HTMLElement"),
        ConstructorKind::Html,
        0,
    ),
    entry(
        "HTMLTimeElement",
        Some("HTMLElement"),
        ConstructorKind::Html,
        0,
    ),
    entry(
        "HTMLTitleElement",
        Some("HTMLElement"),
        ConstructorKind::Html,
        0,
    ),
    entry(
        "HTMLTrackElement",
        Some("HTMLElement"),
        ConstructorKind::Html,
        0,
    ),
    entry(
        "HTMLUListElement",
        Some("HTMLElement"),
        ConstructorKind::Html,
        0,
    ),
    entry(
        "HTMLUnknownElement",
        Some("HTMLElement"),
        ConstructorKind::Illegal,
        0,
    ),
    entry(
        "HTMLVideoElement",
        Some("HTMLMediaElement"),
        ConstructorKind::Html,
        0,
    ),
    entry(
        "MathMLElement",
        Some("Element"),
        ConstructorKind::Illegal,
        0,
    ),
    entry(
        "ProcessingInstruction",
        Some("CharacterData"),
        ConstructorKind::ProcessingInstruction,
        1,
    ),
    entry("SVGElement", Some("Element"), ConstructorKind::Illegal, 0),
    entry(
        "SVGGraphicsElement",
        Some("SVGElement"),
        ConstructorKind::Illegal,
        0,
    ),
    entry(
        "SVGAElement",
        Some("SVGGraphicsElement"),
        ConstructorKind::Illegal,
        0,
    ),
    entry(
        "SVGAnimationElement",
        Some("SVGElement"),
        ConstructorKind::Illegal,
        0,
    ),
    entry(
        "SVGAnimateElement",
        Some("SVGAnimationElement"),
        ConstructorKind::Illegal,
        0,
    ),
    entry(
        "SVGAnimateMotionElement",
        Some("SVGAnimationElement"),
        ConstructorKind::Illegal,
        0,
    ),
    entry(
        "SVGAnimateTransformElement",
        Some("SVGAnimationElement"),
        ConstructorKind::Illegal,
        0,
    ),
    entry(
        "SVGGeometryElement",
        Some("SVGGraphicsElement"),
        ConstructorKind::Illegal,
        0,
    ),
    entry(
        "SVGCircleElement",
        Some("SVGGeometryElement"),
        ConstructorKind::Illegal,
        0,
    ),
    entry(
        "SVGClipPathElement",
        Some("SVGElement"),
        ConstructorKind::Illegal,
        0,
    ),
    entry(
        "SVGComponentTransferFunctionElement",
        Some("SVGElement"),
        ConstructorKind::Illegal,
        0,
    ),
    entry(
        "SVGDefsElement",
        Some("SVGGraphicsElement"),
        ConstructorKind::Illegal,
        0,
    ),
    entry(
        "SVGDescElement",
        Some("SVGElement"),
        ConstructorKind::Illegal,
        0,
    ),
    entry(
        "SVGEllipseElement",
        Some("SVGGeometryElement"),
        ConstructorKind::Illegal,
        0,
    ),
    entry(
        "SVGFEBlendElement",
        Some("SVGElement"),
        ConstructorKind::Illegal,
        0,
    ),
    entry(
        "SVGFEColorMatrixElement",
        Some("SVGElement"),
        ConstructorKind::Illegal,
        0,
    ),
    entry(
        "SVGFEComponentTransferElement",
        Some("SVGElement"),
        ConstructorKind::Illegal,
        0,
    ),
    entry(
        "SVGFECompositeElement",
        Some("SVGElement"),
        ConstructorKind::Illegal,
        0,
    ),
    entry(
        "SVGFEConvolveMatrixElement",
        Some("SVGElement"),
        ConstructorKind::Illegal,
        0,
    ),
    entry(
        "SVGFEDiffuseLightingElement",
        Some("SVGElement"),
        ConstructorKind::Illegal,
        0,
    ),
    entry(
        "SVGFEDisplacementMapElement",
        Some("SVGElement"),
        ConstructorKind::Illegal,
        0,
    ),
    entry(
        "SVGFEDistantLightElement",
        Some("SVGElement"),
        ConstructorKind::Illegal,
        0,
    ),
    entry(
        "SVGFEDropShadowElement",
        Some("SVGElement"),
        ConstructorKind::Illegal,
        0,
    ),
    entry(
        "SVGFEFloodElement",
        Some("SVGElement"),
        ConstructorKind::Illegal,
        0,
    ),
    entry(
        "SVGFEFuncAElement",
        Some("SVGComponentTransferFunctionElement"),
        ConstructorKind::Illegal,
        0,
    ),
    entry(
        "SVGFEFuncBElement",
        Some("SVGComponentTransferFunctionElement"),
        ConstructorKind::Illegal,
        0,
    ),
    entry(
        "SVGFEFuncGElement",
        Some("SVGComponentTransferFunctionElement"),
        ConstructorKind::Illegal,
        0,
    ),
    entry(
        "SVGFEFuncRElement",
        Some("SVGComponentTransferFunctionElement"),
        ConstructorKind::Illegal,
        0,
    ),
    entry(
        "SVGFEGaussianBlurElement",
        Some("SVGElement"),
        ConstructorKind::Illegal,
        0,
    ),
    entry(
        "SVGFEImageElement",
        Some("SVGElement"),
        ConstructorKind::Illegal,
        0,
    ),
    entry(
        "SVGFEMergeElement",
        Some("SVGElement"),
        ConstructorKind::Illegal,
        0,
    ),
    entry(
        "SVGFEMergeNodeElement",
        Some("SVGElement"),
        ConstructorKind::Illegal,
        0,
    ),
    entry(
        "SVGFEMorphologyElement",
        Some("SVGElement"),
        ConstructorKind::Illegal,
        0,
    ),
    entry(
        "SVGFEOffsetElement",
        Some("SVGElement"),
        ConstructorKind::Illegal,
        0,
    ),
    entry(
        "SVGFEPointLightElement",
        Some("SVGElement"),
        ConstructorKind::Illegal,
        0,
    ),
    entry(
        "SVGFESpecularLightingElement",
        Some("SVGElement"),
        ConstructorKind::Illegal,
        0,
    ),
    entry(
        "SVGFESpotLightElement",
        Some("SVGElement"),
        ConstructorKind::Illegal,
        0,
    ),
    entry(
        "SVGFETileElement",
        Some("SVGElement"),
        ConstructorKind::Illegal,
        0,
    ),
    entry(
        "SVGFETurbulenceElement",
        Some("SVGElement"),
        ConstructorKind::Illegal,
        0,
    ),
    entry(
        "SVGFilterElement",
        Some("SVGElement"),
        ConstructorKind::Illegal,
        0,
    ),
    entry(
        "SVGForeignObjectElement",
        Some("SVGGraphicsElement"),
        ConstructorKind::Illegal,
        0,
    ),
    entry(
        "SVGGElement",
        Some("SVGGraphicsElement"),
        ConstructorKind::Illegal,
        0,
    ),
    entry(
        "SVGGradientElement",
        Some("SVGElement"),
        ConstructorKind::Illegal,
        0,
    ),
    entry(
        "SVGImageElement",
        Some("SVGGraphicsElement"),
        ConstructorKind::Illegal,
        0,
    ),
    entry(
        "SVGLineElement",
        Some("SVGGeometryElement"),
        ConstructorKind::Illegal,
        0,
    ),
    entry(
        "SVGLinearGradientElement",
        Some("SVGGradientElement"),
        ConstructorKind::Illegal,
        0,
    ),
    entry(
        "SVGMPathElement",
        Some("SVGElement"),
        ConstructorKind::Illegal,
        0,
    ),
    entry(
        "SVGMarkerElement",
        Some("SVGElement"),
        ConstructorKind::Illegal,
        0,
    ),
    entry(
        "SVGMaskElement",
        Some("SVGElement"),
        ConstructorKind::Illegal,
        0,
    ),
    entry(
        "SVGMetadataElement",
        Some("SVGElement"),
        ConstructorKind::Illegal,
        0,
    ),
    entry(
        "SVGPathElement",
        Some("SVGGeometryElement"),
        ConstructorKind::Illegal,
        0,
    ),
    entry(
        "SVGPatternElement",
        Some("SVGElement"),
        ConstructorKind::Illegal,
        0,
    ),
    entry(
        "SVGPolygonElement",
        Some("SVGGeometryElement"),
        ConstructorKind::Illegal,
        0,
    ),
    entry(
        "SVGPolylineElement",
        Some("SVGGeometryElement"),
        ConstructorKind::Illegal,
        0,
    ),
    entry(
        "SVGRadialGradientElement",
        Some("SVGGradientElement"),
        ConstructorKind::Illegal,
        0,
    ),
    entry(
        "SVGRectElement",
        Some("SVGGeometryElement"),
        ConstructorKind::Illegal,
        0,
    ),
    entry(
        "SVGSVGElement",
        Some("SVGGraphicsElement"),
        ConstructorKind::Illegal,
        0,
    ),
    entry(
        "SVGScriptElement",
        Some("SVGElement"),
        ConstructorKind::Illegal,
        0,
    ),
    entry(
        "SVGSetElement",
        Some("SVGAnimationElement"),
        ConstructorKind::Illegal,
        0,
    ),
    entry(
        "SVGStopElement",
        Some("SVGElement"),
        ConstructorKind::Illegal,
        0,
    ),
    entry(
        "SVGStyleElement",
        Some("SVGElement"),
        ConstructorKind::Illegal,
        0,
    ),
    entry(
        "SVGSwitchElement",
        Some("SVGGraphicsElement"),
        ConstructorKind::Illegal,
        0,
    ),
    entry(
        "SVGSymbolElement",
        Some("SVGGraphicsElement"),
        ConstructorKind::Illegal,
        0,
    ),
    entry(
        "SVGTextContentElement",
        Some("SVGGraphicsElement"),
        ConstructorKind::Illegal,
        0,
    ),
    entry(
        "SVGTextPositioningElement",
        Some("SVGTextContentElement"),
        ConstructorKind::Illegal,
        0,
    ),
    entry(
        "SVGTSpanElement",
        Some("SVGTextPositioningElement"),
        ConstructorKind::Illegal,
        0,
    ),
    entry(
        "SVGTextElement",
        Some("SVGTextPositioningElement"),
        ConstructorKind::Illegal,
        0,
    ),
    entry(
        "SVGTextPathElement",
        Some("SVGTextContentElement"),
        ConstructorKind::Illegal,
        0,
    ),
    entry(
        "SVGTitleElement",
        Some("SVGElement"),
        ConstructorKind::Illegal,
        0,
    ),
    entry(
        "SVGUseElement",
        Some("SVGGraphicsElement"),
        ConstructorKind::Illegal,
        0,
    ),
    entry(
        "SVGViewElement",
        Some("SVGElement"),
        ConstructorKind::Illegal,
        0,
    ),
    entry("Text", Some("CharacterData"), ConstructorKind::Text, 0),
];

// A separate sorted index permits bounded lookup without changing installation
// order or allocating an owned map. Each stored name is the descriptor's name.
// Literal parent indices in INTERFACES order, validated against parent names.
// Installer callers pay one indexed access instead of a repeated name search.
const PARENT_INDICES: [Option<usize>; INTERFACES.len()] = [
    None,      // EventTarget
    Some(0),   // Node
    Some(1),   // CharacterData
    Some(2),   // Comment
    Some(1),   // Document
    Some(1),   // DocumentFragment
    Some(1),   // DocumentType
    Some(1),   // Element
    Some(7),   // HTMLElement
    Some(8),   // HTMLAnchorElement
    Some(8),   // HTMLAreaElement
    Some(8),   // HTMLMediaElement
    Some(11),  // HTMLAudioElement
    Some(8),   // HTMLBRElement
    Some(8),   // HTMLBaseElement
    Some(8),   // HTMLBodyElement
    Some(8),   // HTMLButtonElement
    Some(8),   // HTMLCanvasElement
    Some(8),   // HTMLDListElement
    Some(8),   // HTMLDataElement
    Some(8),   // HTMLDataListElement
    Some(8),   // HTMLDetailsElement
    Some(8),   // HTMLDialogElement
    Some(8),   // HTMLDirectoryElement
    Some(8),   // HTMLDivElement
    Some(8),   // HTMLEmbedElement
    Some(8),   // HTMLFieldSetElement
    Some(8),   // HTMLFontElement
    Some(8),   // HTMLFormElement
    Some(8),   // HTMLFrameElement
    Some(8),   // HTMLFrameSetElement
    Some(8),   // HTMLHRElement
    Some(8),   // HTMLHeadElement
    Some(8),   // HTMLHeadingElement
    Some(8),   // HTMLHtmlElement
    Some(8),   // HTMLIFrameElement
    Some(8),   // HTMLImageElement
    Some(8),   // HTMLInputElement
    Some(8),   // HTMLLIElement
    Some(8),   // HTMLLabelElement
    Some(8),   // HTMLLegendElement
    Some(8),   // HTMLLinkElement
    Some(8),   // HTMLMapElement
    Some(8),   // HTMLMarqueeElement
    Some(8),   // HTMLMenuElement
    Some(8),   // HTMLMetaElement
    Some(8),   // HTMLMeterElement
    Some(8),   // HTMLModElement
    Some(8),   // HTMLOListElement
    Some(8),   // HTMLObjectElement
    Some(8),   // HTMLOptGroupElement
    Some(8),   // HTMLOptionElement
    Some(8),   // HTMLOutputElement
    Some(8),   // HTMLParagraphElement
    Some(8),   // HTMLParamElement
    Some(8),   // HTMLPictureElement
    Some(8),   // HTMLPreElement
    Some(8),   // HTMLProgressElement
    Some(8),   // HTMLQuoteElement
    Some(8),   // HTMLScriptElement
    Some(8),   // HTMLSelectElement
    Some(8),   // HTMLSelectedContentElement
    Some(8),   // HTMLSlotElement
    Some(8),   // HTMLSourceElement
    Some(8),   // HTMLSpanElement
    Some(8),   // HTMLStyleElement
    Some(8),   // HTMLTableCaptionElement
    Some(8),   // HTMLTableCellElement
    Some(8),   // HTMLTableColElement
    Some(8),   // HTMLTableElement
    Some(8),   // HTMLTableRowElement
    Some(8),   // HTMLTableSectionElement
    Some(8),   // HTMLTemplateElement
    Some(8),   // HTMLTextAreaElement
    Some(8),   // HTMLTimeElement
    Some(8),   // HTMLTitleElement
    Some(8),   // HTMLTrackElement
    Some(8),   // HTMLUListElement
    Some(8),   // HTMLUnknownElement
    Some(11),  // HTMLVideoElement
    Some(7),   // MathMLElement
    Some(2),   // ProcessingInstruction
    Some(7),   // SVGElement
    Some(82),  // SVGGraphicsElement
    Some(83),  // SVGAElement
    Some(82),  // SVGAnimationElement
    Some(85),  // SVGAnimateElement
    Some(85),  // SVGAnimateMotionElement
    Some(85),  // SVGAnimateTransformElement
    Some(83),  // SVGGeometryElement
    Some(89),  // SVGCircleElement
    Some(82),  // SVGClipPathElement
    Some(82),  // SVGComponentTransferFunctionElement
    Some(83),  // SVGDefsElement
    Some(82),  // SVGDescElement
    Some(89),  // SVGEllipseElement
    Some(82),  // SVGFEBlendElement
    Some(82),  // SVGFEColorMatrixElement
    Some(82),  // SVGFEComponentTransferElement
    Some(82),  // SVGFECompositeElement
    Some(82),  // SVGFEConvolveMatrixElement
    Some(82),  // SVGFEDiffuseLightingElement
    Some(82),  // SVGFEDisplacementMapElement
    Some(82),  // SVGFEDistantLightElement
    Some(82),  // SVGFEDropShadowElement
    Some(82),  // SVGFEFloodElement
    Some(92),  // SVGFEFuncAElement
    Some(92),  // SVGFEFuncBElement
    Some(92),  // SVGFEFuncGElement
    Some(92),  // SVGFEFuncRElement
    Some(82),  // SVGFEGaussianBlurElement
    Some(82),  // SVGFEImageElement
    Some(82),  // SVGFEMergeElement
    Some(82),  // SVGFEMergeNodeElement
    Some(82),  // SVGFEMorphologyElement
    Some(82),  // SVGFEOffsetElement
    Some(82),  // SVGFEPointLightElement
    Some(82),  // SVGFESpecularLightingElement
    Some(82),  // SVGFESpotLightElement
    Some(82),  // SVGFETileElement
    Some(82),  // SVGFETurbulenceElement
    Some(82),  // SVGFilterElement
    Some(83),  // SVGForeignObjectElement
    Some(83),  // SVGGElement
    Some(82),  // SVGGradientElement
    Some(83),  // SVGImageElement
    Some(89),  // SVGLineElement
    Some(124), // SVGLinearGradientElement
    Some(82),  // SVGMPathElement
    Some(82),  // SVGMarkerElement
    Some(82),  // SVGMaskElement
    Some(82),  // SVGMetadataElement
    Some(89),  // SVGPathElement
    Some(82),  // SVGPatternElement
    Some(89),  // SVGPolygonElement
    Some(89),  // SVGPolylineElement
    Some(124), // SVGRadialGradientElement
    Some(89),  // SVGRectElement
    Some(83),  // SVGSVGElement
    Some(82),  // SVGScriptElement
    Some(85),  // SVGSetElement
    Some(82),  // SVGStopElement
    Some(82),  // SVGStyleElement
    Some(83),  // SVGSwitchElement
    Some(83),  // SVGSymbolElement
    Some(83),  // SVGTextContentElement
    Some(145), // SVGTextPositioningElement
    Some(146), // SVGTSpanElement
    Some(146), // SVGTextElement
    Some(145), // SVGTextPathElement
    Some(82),  // SVGTitleElement
    Some(83),  // SVGUseElement
    Some(82),  // SVGViewElement
    Some(2),   // Text
];

const INTERFACE_INDEX: &[(&str, usize)] = &[
    ("CharacterData", 2),
    ("Comment", 3),
    ("Document", 4),
    ("DocumentFragment", 5),
    ("DocumentType", 6),
    ("Element", 7),
    ("EventTarget", 0),
    ("HTMLAnchorElement", 9),
    ("HTMLAreaElement", 10),
    ("HTMLAudioElement", 12),
    ("HTMLBRElement", 13),
    ("HTMLBaseElement", 14),
    ("HTMLBodyElement", 15),
    ("HTMLButtonElement", 16),
    ("HTMLCanvasElement", 17),
    ("HTMLDListElement", 18),
    ("HTMLDataElement", 19),
    ("HTMLDataListElement", 20),
    ("HTMLDetailsElement", 21),
    ("HTMLDialogElement", 22),
    ("HTMLDirectoryElement", 23),
    ("HTMLDivElement", 24),
    ("HTMLElement", 8),
    ("HTMLEmbedElement", 25),
    ("HTMLFieldSetElement", 26),
    ("HTMLFontElement", 27),
    ("HTMLFormElement", 28),
    ("HTMLFrameElement", 29),
    ("HTMLFrameSetElement", 30),
    ("HTMLHRElement", 31),
    ("HTMLHeadElement", 32),
    ("HTMLHeadingElement", 33),
    ("HTMLHtmlElement", 34),
    ("HTMLIFrameElement", 35),
    ("HTMLImageElement", 36),
    ("HTMLInputElement", 37),
    ("HTMLLIElement", 38),
    ("HTMLLabelElement", 39),
    ("HTMLLegendElement", 40),
    ("HTMLLinkElement", 41),
    ("HTMLMapElement", 42),
    ("HTMLMarqueeElement", 43),
    ("HTMLMediaElement", 11),
    ("HTMLMenuElement", 44),
    ("HTMLMetaElement", 45),
    ("HTMLMeterElement", 46),
    ("HTMLModElement", 47),
    ("HTMLOListElement", 48),
    ("HTMLObjectElement", 49),
    ("HTMLOptGroupElement", 50),
    ("HTMLOptionElement", 51),
    ("HTMLOutputElement", 52),
    ("HTMLParagraphElement", 53),
    ("HTMLParamElement", 54),
    ("HTMLPictureElement", 55),
    ("HTMLPreElement", 56),
    ("HTMLProgressElement", 57),
    ("HTMLQuoteElement", 58),
    ("HTMLScriptElement", 59),
    ("HTMLSelectElement", 60),
    ("HTMLSelectedContentElement", 61),
    ("HTMLSlotElement", 62),
    ("HTMLSourceElement", 63),
    ("HTMLSpanElement", 64),
    ("HTMLStyleElement", 65),
    ("HTMLTableCaptionElement", 66),
    ("HTMLTableCellElement", 67),
    ("HTMLTableColElement", 68),
    ("HTMLTableElement", 69),
    ("HTMLTableRowElement", 70),
    ("HTMLTableSectionElement", 71),
    ("HTMLTemplateElement", 72),
    ("HTMLTextAreaElement", 73),
    ("HTMLTimeElement", 74),
    ("HTMLTitleElement", 75),
    ("HTMLTrackElement", 76),
    ("HTMLUListElement", 77),
    ("HTMLUnknownElement", 78),
    ("HTMLVideoElement", 79),
    ("MathMLElement", 80),
    ("Node", 1),
    ("ProcessingInstruction", 81),
    ("SVGAElement", 84),
    ("SVGAnimateElement", 86),
    ("SVGAnimateMotionElement", 87),
    ("SVGAnimateTransformElement", 88),
    ("SVGAnimationElement", 85),
    ("SVGCircleElement", 90),
    ("SVGClipPathElement", 91),
    ("SVGComponentTransferFunctionElement", 92),
    ("SVGDefsElement", 93),
    ("SVGDescElement", 94),
    ("SVGElement", 82),
    ("SVGEllipseElement", 95),
    ("SVGFEBlendElement", 96),
    ("SVGFEColorMatrixElement", 97),
    ("SVGFEComponentTransferElement", 98),
    ("SVGFECompositeElement", 99),
    ("SVGFEConvolveMatrixElement", 100),
    ("SVGFEDiffuseLightingElement", 101),
    ("SVGFEDisplacementMapElement", 102),
    ("SVGFEDistantLightElement", 103),
    ("SVGFEDropShadowElement", 104),
    ("SVGFEFloodElement", 105),
    ("SVGFEFuncAElement", 106),
    ("SVGFEFuncBElement", 107),
    ("SVGFEFuncGElement", 108),
    ("SVGFEFuncRElement", 109),
    ("SVGFEGaussianBlurElement", 110),
    ("SVGFEImageElement", 111),
    ("SVGFEMergeElement", 112),
    ("SVGFEMergeNodeElement", 113),
    ("SVGFEMorphologyElement", 114),
    ("SVGFEOffsetElement", 115),
    ("SVGFEPointLightElement", 116),
    ("SVGFESpecularLightingElement", 117),
    ("SVGFESpotLightElement", 118),
    ("SVGFETileElement", 119),
    ("SVGFETurbulenceElement", 120),
    ("SVGFilterElement", 121),
    ("SVGForeignObjectElement", 122),
    ("SVGGElement", 123),
    ("SVGGeometryElement", 89),
    ("SVGGradientElement", 124),
    ("SVGGraphicsElement", 83),
    ("SVGImageElement", 125),
    ("SVGLineElement", 126),
    ("SVGLinearGradientElement", 127),
    ("SVGMPathElement", 128),
    ("SVGMarkerElement", 129),
    ("SVGMaskElement", 130),
    ("SVGMetadataElement", 131),
    ("SVGPathElement", 132),
    ("SVGPatternElement", 133),
    ("SVGPolygonElement", 134),
    ("SVGPolylineElement", 135),
    ("SVGRadialGradientElement", 136),
    ("SVGRectElement", 137),
    ("SVGSVGElement", 138),
    ("SVGScriptElement", 139),
    ("SVGSetElement", 140),
    ("SVGStopElement", 141),
    ("SVGStyleElement", 142),
    ("SVGSwitchElement", 143),
    ("SVGSymbolElement", 144),
    ("SVGTSpanElement", 147),
    ("SVGTextContentElement", 145),
    ("SVGTextElement", 148),
    ("SVGTextPathElement", 149),
    ("SVGTextPositioningElement", 146),
    ("SVGTitleElement", 150),
    ("SVGUseElement", 151),
    ("SVGViewElement", 152),
    ("Text", 153),
];

// Includes current HTML elements, obsolete-defined interfaces, the explicit
// obsolete fallbacks, and the eight reserved custom-element names.
const HTML_NAMES: &[(&str, usize)] = &[
    ("a", 9),                 // HTMLAnchorElement
    ("abbr", 8),              // HTMLElement
    ("acronym", 8),           // HTMLElement
    ("address", 8),           // HTMLElement
    ("annotation-xml", 78),   // HTMLUnknownElement
    ("applet", 78),           // HTMLUnknownElement
    ("area", 10),             // HTMLAreaElement
    ("article", 8),           // HTMLElement
    ("aside", 8),             // HTMLElement
    ("audio", 12),            // HTMLAudioElement
    ("b", 8),                 // HTMLElement
    ("base", 14),             // HTMLBaseElement
    ("basefont", 8),          // HTMLElement
    ("bdi", 8),               // HTMLElement
    ("bdo", 8),               // HTMLElement
    ("bgsound", 78),          // HTMLUnknownElement
    ("big", 8),               // HTMLElement
    ("blink", 78),            // HTMLUnknownElement
    ("blockquote", 58),       // HTMLQuoteElement
    ("body", 15),             // HTMLBodyElement
    ("br", 13),               // HTMLBRElement
    ("button", 16),           // HTMLButtonElement
    ("canvas", 17),           // HTMLCanvasElement
    ("caption", 66),          // HTMLTableCaptionElement
    ("center", 8),            // HTMLElement
    ("cite", 8),              // HTMLElement
    ("code", 8),              // HTMLElement
    ("col", 68),              // HTMLTableColElement
    ("colgroup", 68),         // HTMLTableColElement
    ("color-profile", 78),    // HTMLUnknownElement
    ("data", 19),             // HTMLDataElement
    ("datalist", 20),         // HTMLDataListElement
    ("dd", 8),                // HTMLElement
    ("del", 47),              // HTMLModElement
    ("details", 21),          // HTMLDetailsElement
    ("dfn", 8),               // HTMLElement
    ("dialog", 22),           // HTMLDialogElement
    ("dir", 23),              // HTMLDirectoryElement
    ("div", 24),              // HTMLDivElement
    ("dl", 18),               // HTMLDListElement
    ("dt", 8),                // HTMLElement
    ("em", 8),                // HTMLElement
    ("embed", 25),            // HTMLEmbedElement
    ("fieldset", 26),         // HTMLFieldSetElement
    ("figcaption", 8),        // HTMLElement
    ("figure", 8),            // HTMLElement
    ("font", 27),             // HTMLFontElement
    ("font-face", 78),        // HTMLUnknownElement
    ("font-face-format", 78), // HTMLUnknownElement
    ("font-face-name", 78),   // HTMLUnknownElement
    ("font-face-src", 78),    // HTMLUnknownElement
    ("font-face-uri", 78),    // HTMLUnknownElement
    ("footer", 8),            // HTMLElement
    ("form", 28),             // HTMLFormElement
    ("frame", 29),            // HTMLFrameElement
    ("frameset", 30),         // HTMLFrameSetElement
    ("h1", 33),               // HTMLHeadingElement
    ("h2", 33),               // HTMLHeadingElement
    ("h3", 33),               // HTMLHeadingElement
    ("h4", 33),               // HTMLHeadingElement
    ("h5", 33),               // HTMLHeadingElement
    ("h6", 33),               // HTMLHeadingElement
    ("head", 32),             // HTMLHeadElement
    ("header", 8),            // HTMLElement
    ("hgroup", 8),            // HTMLElement
    ("hr", 31),               // HTMLHRElement
    ("html", 34),             // HTMLHtmlElement
    ("i", 8),                 // HTMLElement
    ("iframe", 35),           // HTMLIFrameElement
    ("img", 36),              // HTMLImageElement
    ("input", 37),            // HTMLInputElement
    ("ins", 47),              // HTMLModElement
    ("isindex", 78),          // HTMLUnknownElement
    ("kbd", 8),               // HTMLElement
    ("keygen", 78),           // HTMLUnknownElement
    ("label", 39),            // HTMLLabelElement
    ("legend", 40),           // HTMLLegendElement
    ("li", 38),               // HTMLLIElement
    ("link", 41),             // HTMLLinkElement
    ("listing", 56),          // HTMLPreElement
    ("main", 8),              // HTMLElement
    ("map", 42),              // HTMLMapElement
    ("mark", 8),              // HTMLElement
    ("marquee", 43),          // HTMLMarqueeElement
    ("menu", 44),             // HTMLMenuElement
    ("meta", 45),             // HTMLMetaElement
    ("meter", 46),            // HTMLMeterElement
    ("missing-glyph", 78),    // HTMLUnknownElement
    ("multicol", 78),         // HTMLUnknownElement
    ("nav", 8),               // HTMLElement
    ("nextid", 78),           // HTMLUnknownElement
    ("nobr", 8),              // HTMLElement
    ("noembed", 8),           // HTMLElement
    ("noframes", 8),          // HTMLElement
    ("noscript", 8),          // HTMLElement
    ("object", 49),           // HTMLObjectElement
    ("ol", 48),               // HTMLOListElement
    ("optgroup", 50),         // HTMLOptGroupElement
    ("option", 51),           // HTMLOptionElement
    ("output", 52),           // HTMLOutputElement
    ("p", 53),                // HTMLParagraphElement
    ("param", 54),            // HTMLParamElement
    ("picture", 55),          // HTMLPictureElement
    ("plaintext", 8),         // HTMLElement
    ("pre", 56),              // HTMLPreElement
    ("progress", 57),         // HTMLProgressElement
    ("q", 58),                // HTMLQuoteElement
    ("rb", 8),                // HTMLElement
    ("rp", 8),                // HTMLElement
    ("rt", 8),                // HTMLElement
    ("rtc", 8),               // HTMLElement
    ("ruby", 8),              // HTMLElement
    ("s", 8),                 // HTMLElement
    ("samp", 8),              // HTMLElement
    ("script", 59),           // HTMLScriptElement
    ("search", 8),            // HTMLElement
    ("section", 8),           // HTMLElement
    ("select", 60),           // HTMLSelectElement
    ("selectedcontent", 61),  // HTMLSelectedContentElement
    ("slot", 62),             // HTMLSlotElement
    ("small", 8),             // HTMLElement
    ("source", 63),           // HTMLSourceElement
    ("spacer", 78),           // HTMLUnknownElement
    ("span", 64),             // HTMLSpanElement
    ("strike", 8),            // HTMLElement
    ("strong", 8),            // HTMLElement
    ("style", 65),            // HTMLStyleElement
    ("sub", 8),               // HTMLElement
    ("summary", 8),           // HTMLElement
    ("sup", 8),               // HTMLElement
    ("table", 69),            // HTMLTableElement
    ("tbody", 71),            // HTMLTableSectionElement
    ("td", 67),               // HTMLTableCellElement
    ("template", 72),         // HTMLTemplateElement
    ("textarea", 73),         // HTMLTextAreaElement
    ("tfoot", 71),            // HTMLTableSectionElement
    ("th", 67),               // HTMLTableCellElement
    ("thead", 71),            // HTMLTableSectionElement
    ("time", 74),             // HTMLTimeElement
    ("title", 75),            // HTMLTitleElement
    ("tr", 70),               // HTMLTableRowElement
    ("track", 76),            // HTMLTrackElement
    ("tt", 8),                // HTMLElement
    ("u", 8),                 // HTMLElement
    ("ul", 77),               // HTMLUListElement
    ("var", 8),               // HTMLElement
    ("video", 79),            // HTMLVideoElement
    ("wbr", 8),               // HTMLElement
    ("xmp", 56),              // HTMLPreElement
];

const SVG_NAMES: &[(&str, usize)] = &[
    ("a", 84),                   // SVGAElement
    ("animate", 86),             // SVGAnimateElement
    ("animateMotion", 87),       // SVGAnimateMotionElement
    ("animateTransform", 88),    // SVGAnimateTransformElement
    ("circle", 90),              // SVGCircleElement
    ("clipPath", 91),            // SVGClipPathElement
    ("defs", 93),                // SVGDefsElement
    ("desc", 94),                // SVGDescElement
    ("ellipse", 95),             // SVGEllipseElement
    ("feBlend", 96),             // SVGFEBlendElement
    ("feColorMatrix", 97),       // SVGFEColorMatrixElement
    ("feComponentTransfer", 98), // SVGFEComponentTransferElement
    ("feComposite", 99),         // SVGFECompositeElement
    ("feConvolveMatrix", 100),   // SVGFEConvolveMatrixElement
    ("feDiffuseLighting", 101),  // SVGFEDiffuseLightingElement
    ("feDisplacementMap", 102),  // SVGFEDisplacementMapElement
    ("feDistantLight", 103),     // SVGFEDistantLightElement
    ("feDropShadow", 104),       // SVGFEDropShadowElement
    ("feFlood", 105),            // SVGFEFloodElement
    ("feFuncA", 106),            // SVGFEFuncAElement
    ("feFuncB", 107),            // SVGFEFuncBElement
    ("feFuncG", 108),            // SVGFEFuncGElement
    ("feFuncR", 109),            // SVGFEFuncRElement
    ("feGaussianBlur", 110),     // SVGFEGaussianBlurElement
    ("feImage", 111),            // SVGFEImageElement
    ("feMerge", 112),            // SVGFEMergeElement
    ("feMergeNode", 113),        // SVGFEMergeNodeElement
    ("feMorphology", 114),       // SVGFEMorphologyElement
    ("feOffset", 115),           // SVGFEOffsetElement
    ("fePointLight", 116),       // SVGFEPointLightElement
    ("feSpecularLighting", 117), // SVGFESpecularLightingElement
    ("feSpotLight", 118),        // SVGFESpotLightElement
    ("feTile", 119),             // SVGFETileElement
    ("feTurbulence", 120),       // SVGFETurbulenceElement
    ("filter", 121),             // SVGFilterElement
    ("foreignObject", 122),      // SVGForeignObjectElement
    ("g", 123),                  // SVGGElement
    ("image", 125),              // SVGImageElement
    ("line", 126),               // SVGLineElement
    ("linearGradient", 127),     // SVGLinearGradientElement
    ("marker", 129),             // SVGMarkerElement
    ("mask", 130),               // SVGMaskElement
    ("metadata", 131),           // SVGMetadataElement
    ("mpath", 128),              // SVGMPathElement
    ("path", 132),               // SVGPathElement
    ("pattern", 133),            // SVGPatternElement
    ("polygon", 134),            // SVGPolygonElement
    ("polyline", 135),           // SVGPolylineElement
    ("radialGradient", 136),     // SVGRadialGradientElement
    ("rect", 137),               // SVGRectElement
    ("script", 139),             // SVGScriptElement
    ("set", 140),                // SVGSetElement
    ("stop", 141),               // SVGStopElement
    ("style", 142),              // SVGStyleElement
    ("svg", 138),                // SVGSVGElement
    ("switch", 143),             // SVGSwitchElement
    ("symbol", 144),             // SVGSymbolElement
    ("text", 148),               // SVGTextElement
    ("textPath", 149),           // SVGTextPathElement
    ("title", 150),              // SVGTitleElement
    ("tspan", 147),              // SVGTSpanElement
    ("use", 151),                // SVGUseElement
    ("view", 152),               // SVGViewElement
];

const fn max_name_bytes<T>(table: &[(&str, T)]) -> usize {
    let mut maximum = 0;
    let mut at = 0;
    while at < table.len() {
        let length = table[at].0.len();
        if length > maximum {
            maximum = length;
        }
        at += 1;
    }
    maximum
}

const HTML_MAX_NAME: usize = max_name_bytes(HTML_NAMES);
const SVG_MAX_NAME: usize = max_name_bytes(SVG_NAMES);
const INTERFACE_MAX_NAME: usize = max_name_bytes(INTERFACE_INDEX);

fn lookup<T: Copy>(table: &[(&str, T)], name: &str) -> Option<T> {
    let mut low = 0;
    let mut high = table.len();
    while low < high {
        let middle = low + (high - low) / 2;
        match table[middle].0.cmp(name) {
            Ordering::Less => low = middle + 1,
            Ordering::Greater => high = middle,
            Ordering::Equal => return Some(table[middle].1),
        }
    }
    None
}

// floor(log2(count)) + 1 bounds comparisons in the explicit binary search.
// Each comparison reserves its possible byte prefix plus 9 fixed units for
// length/order decisions and loop bookkeeping. No name scan happens here.
fn lookup_work(count: usize, maximum: usize, name_bytes: usize) -> usize {
    let comparisons = usize::BITS as usize - count.leading_zeros() as usize;
    comparisons * (name_bytes.min(maximum) + 9)
}

/// Prepay before interface_index(name) or interface(name) for a runtime name.
/// This is separate from caller-owned prototype/native registry lookup fees.
pub(super) fn interface_lookup_work(name_bytes: usize) -> usize {
    lookup_work(INTERFACE_INDEX.len(), INTERFACE_MAX_NAME, name_bytes)
}

/// Index into INTERFACES, using the same single prepaid name lookup.
pub(super) fn interface_index(name: &str) -> Option<usize> {
    lookup(INTERFACE_INDEX, name)
}

pub(super) fn interface(name: &str) -> Option<&'static Interface> {
    interface_index(name).map(|index| &INTERFACES[index])
}

/// Existing lexicographic interface-name order; no sorting or allocation.
/// Callers account for each visited index and any retained copy.
pub(super) fn interface_order() -> impl Iterator<Item = usize> {
    INTERFACE_INDEX.iter().map(|&(_, index)| index)
}

/// Fixed indexed access. None denotes the root interface or an invalid index.
/// Callers using a known interface index pay a fixed access, with no name scan.
pub(super) fn parent_index(index: usize) -> Option<usize> {
    PARENT_INDICES.get(index).copied().flatten()
}

fn element_mapping_work(namespace: Namespace, name_bytes: usize) -> Option<usize> {
    match namespace {
        // The custom-name fallback makes one complete byte pass at most:
        // one visit plus one classification unit per byte, prepaid even on a
        // successful ordinary-tag lookup. Checked arithmetic refuses overflow.
        Namespace::Html => name_bytes
            .checked_mul(2)?
            .checked_add(lookup_work(HTML_NAMES.len(), HTML_MAX_NAME, name_bytes))?
            .checked_add(4),
        Namespace::Svg => Some(lookup_work(SVG_NAMES.len(), SVG_MAX_NAME, name_bytes) + 4),
        Namespace::MathMl => Some(4),
    }
}

/// Pure O(1) preflight: pay before node_interface_index or its name wrapper. None means
/// an invalid NodeId or arithmetic overflow, never permission for free lookup.
/// It covers only initial node/tag mapping; it allocates no storage.
pub(super) fn mapping_work(doc: &Document, id: NodeId) -> Option<usize> {
    match &doc.nodes.get(id)?.kind {
        NodeKind::Element(element) => element_mapping_work(element.namespace, element.tag.len()),
        _ => Some(4),
    }
}

// Literal direct targets, checked against independent names in private tests.
const DOCUMENT_INDEX: usize = 4; // Document
const DOCUMENT_FRAGMENT_INDEX: usize = 5; // DocumentFragment
const TEXT_INDEX: usize = 153; // Text
const COMMENT_INDEX: usize = 3; // Comment
const DOCUMENT_TYPE_INDEX: usize = 6; // DocumentType
const PROCESSING_INSTRUCTION_INDEX: usize = 81; // ProcessingInstruction
const HTML_ELEMENT_INDEX: usize = 8; // HTMLElement
const HTML_UNKNOWN_ELEMENT_INDEX: usize = 78; // HTMLUnknownElement
const SVG_ELEMENT_INDEX: usize = 82; // SVGElement
const MATH_ML_ELEMENT_INDEX: usize = 80; // MathMLElement

/// Must follow successful mapping_work prepayment. Uses the stored namespace
/// and local name, without lowercasing, cloning, parsing, callbacks or mutation.
/// Returned indices already identify INTERFACES; no second name search is needed.
pub(super) fn node_interface_index(doc: &Document, id: NodeId) -> Option<usize> {
    Some(match &doc.nodes.get(id)?.kind {
        NodeKind::Document => DOCUMENT_INDEX,
        NodeKind::DocumentFragment { .. } => DOCUMENT_FRAGMENT_INDEX,
        NodeKind::Element(element) => element_interface_index(element.namespace, &element.tag),
        NodeKind::Text(_) => TEXT_INDEX,
        NodeKind::Comment(_) => COMMENT_INDEX,
        NodeKind::Doctype(_) => DOCUMENT_TYPE_INDEX,
        NodeKind::ProcessingInstruction { .. } => PROCESSING_INSTRUCTION_INDEX,
    })
}

/// Name wrapper for independent mapping expectations; uses the same single map.
#[cfg(test)]
pub(super) fn node_interface(doc: &Document, id: NodeId) -> Option<&'static str> {
    node_interface_index(doc, id).map(|index| INTERFACES[index].name)
}

#[cfg(test)]
fn element_interface(namespace: Namespace, name: &str) -> &'static str {
    INTERFACES[element_interface_index(namespace, name)].name
}

fn element_interface_index(namespace: Namespace, name: &str) -> usize {
    match namespace {
        Namespace::Html => lookup(HTML_NAMES, name).unwrap_or_else(|| {
            if custom_name_after_reserved_lookup(name) {
                HTML_ELEMENT_INDEX
            } else {
                HTML_UNKNOWN_ELEMENT_INDEX
            }
        }),
        Namespace::Svg => lookup(SVG_NAMES, name).unwrap_or(SVG_ELEMENT_INDEX),
        Namespace::MathMl => MATH_ML_ELEMENT_INDEX,
    }
}

// Current HTML valid-custom-element-name references DOM's valid-element-local-
// name, rather than the older PCENChar grammar:
// https://html.spec.whatwg.org/multipage/custom-elements.html#valid-custom-element-name
// https://dom.spec.whatwg.org/#valid-element-local-name
// Once the required first lowercase ASCII letter is established, the current
// local-name rule rejects only ASCII whitespace, NULL, '/' and '>'. Also reject
// uppercase ASCII and require a hyphen. UTF-8 bytes >= 128 cannot be an excluded
// ASCII code point, so this single byte pass preserves all represented scalars.
// The eight reserved names have already matched HTML_NAMES.
fn custom_name_after_reserved_lookup(name: &str) -> bool {
    if !name.as_bytes().first().is_some_and(u8::is_ascii_lowercase) {
        return false;
    }
    let mut hyphen = false;
    for byte in name.bytes() {
        if byte.is_ascii_uppercase()
            || matches!(
                byte,
                0 | b'\t' | b'\n' | b'\x0c' | b'\r' | b' ' | b'/' | b'>'
            )
        {
            return false;
        }
        hyphen |= byte == b'-';
    }
    hyphen
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dom::{Element, Node};
    use std::collections::BTreeMap;

    // Retained pre-index tag-to-name expectations, independent of numeric targets.
    const HTML_EXPECTED: &[(&str, &str)] = &[
        ("a", "HTMLAnchorElement"),
        ("abbr", "HTMLElement"),
        ("acronym", "HTMLElement"),
        ("address", "HTMLElement"),
        ("annotation-xml", "HTMLUnknownElement"),
        ("applet", "HTMLUnknownElement"),
        ("area", "HTMLAreaElement"),
        ("article", "HTMLElement"),
        ("aside", "HTMLElement"),
        ("audio", "HTMLAudioElement"),
        ("b", "HTMLElement"),
        ("base", "HTMLBaseElement"),
        ("basefont", "HTMLElement"),
        ("bdi", "HTMLElement"),
        ("bdo", "HTMLElement"),
        ("bgsound", "HTMLUnknownElement"),
        ("big", "HTMLElement"),
        ("blink", "HTMLUnknownElement"),
        ("blockquote", "HTMLQuoteElement"),
        ("body", "HTMLBodyElement"),
        ("br", "HTMLBRElement"),
        ("button", "HTMLButtonElement"),
        ("canvas", "HTMLCanvasElement"),
        ("caption", "HTMLTableCaptionElement"),
        ("center", "HTMLElement"),
        ("cite", "HTMLElement"),
        ("code", "HTMLElement"),
        ("col", "HTMLTableColElement"),
        ("colgroup", "HTMLTableColElement"),
        ("color-profile", "HTMLUnknownElement"),
        ("data", "HTMLDataElement"),
        ("datalist", "HTMLDataListElement"),
        ("dd", "HTMLElement"),
        ("del", "HTMLModElement"),
        ("details", "HTMLDetailsElement"),
        ("dfn", "HTMLElement"),
        ("dialog", "HTMLDialogElement"),
        ("dir", "HTMLDirectoryElement"),
        ("div", "HTMLDivElement"),
        ("dl", "HTMLDListElement"),
        ("dt", "HTMLElement"),
        ("em", "HTMLElement"),
        ("embed", "HTMLEmbedElement"),
        ("fieldset", "HTMLFieldSetElement"),
        ("figcaption", "HTMLElement"),
        ("figure", "HTMLElement"),
        ("font", "HTMLFontElement"),
        ("font-face", "HTMLUnknownElement"),
        ("font-face-format", "HTMLUnknownElement"),
        ("font-face-name", "HTMLUnknownElement"),
        ("font-face-src", "HTMLUnknownElement"),
        ("font-face-uri", "HTMLUnknownElement"),
        ("footer", "HTMLElement"),
        ("form", "HTMLFormElement"),
        ("frame", "HTMLFrameElement"),
        ("frameset", "HTMLFrameSetElement"),
        ("h1", "HTMLHeadingElement"),
        ("h2", "HTMLHeadingElement"),
        ("h3", "HTMLHeadingElement"),
        ("h4", "HTMLHeadingElement"),
        ("h5", "HTMLHeadingElement"),
        ("h6", "HTMLHeadingElement"),
        ("head", "HTMLHeadElement"),
        ("header", "HTMLElement"),
        ("hgroup", "HTMLElement"),
        ("hr", "HTMLHRElement"),
        ("html", "HTMLHtmlElement"),
        ("i", "HTMLElement"),
        ("iframe", "HTMLIFrameElement"),
        ("img", "HTMLImageElement"),
        ("input", "HTMLInputElement"),
        ("ins", "HTMLModElement"),
        ("isindex", "HTMLUnknownElement"),
        ("kbd", "HTMLElement"),
        ("keygen", "HTMLUnknownElement"),
        ("label", "HTMLLabelElement"),
        ("legend", "HTMLLegendElement"),
        ("li", "HTMLLIElement"),
        ("link", "HTMLLinkElement"),
        ("listing", "HTMLPreElement"),
        ("main", "HTMLElement"),
        ("map", "HTMLMapElement"),
        ("mark", "HTMLElement"),
        ("marquee", "HTMLMarqueeElement"),
        ("menu", "HTMLMenuElement"),
        ("meta", "HTMLMetaElement"),
        ("meter", "HTMLMeterElement"),
        ("missing-glyph", "HTMLUnknownElement"),
        ("multicol", "HTMLUnknownElement"),
        ("nav", "HTMLElement"),
        ("nextid", "HTMLUnknownElement"),
        ("nobr", "HTMLElement"),
        ("noembed", "HTMLElement"),
        ("noframes", "HTMLElement"),
        ("noscript", "HTMLElement"),
        ("object", "HTMLObjectElement"),
        ("ol", "HTMLOListElement"),
        ("optgroup", "HTMLOptGroupElement"),
        ("option", "HTMLOptionElement"),
        ("output", "HTMLOutputElement"),
        ("p", "HTMLParagraphElement"),
        ("param", "HTMLParamElement"),
        ("picture", "HTMLPictureElement"),
        ("plaintext", "HTMLElement"),
        ("pre", "HTMLPreElement"),
        ("progress", "HTMLProgressElement"),
        ("q", "HTMLQuoteElement"),
        ("rb", "HTMLElement"),
        ("rp", "HTMLElement"),
        ("rt", "HTMLElement"),
        ("rtc", "HTMLElement"),
        ("ruby", "HTMLElement"),
        ("s", "HTMLElement"),
        ("samp", "HTMLElement"),
        ("script", "HTMLScriptElement"),
        ("search", "HTMLElement"),
        ("section", "HTMLElement"),
        ("select", "HTMLSelectElement"),
        ("selectedcontent", "HTMLSelectedContentElement"),
        ("slot", "HTMLSlotElement"),
        ("small", "HTMLElement"),
        ("source", "HTMLSourceElement"),
        ("spacer", "HTMLUnknownElement"),
        ("span", "HTMLSpanElement"),
        ("strike", "HTMLElement"),
        ("strong", "HTMLElement"),
        ("style", "HTMLStyleElement"),
        ("sub", "HTMLElement"),
        ("summary", "HTMLElement"),
        ("sup", "HTMLElement"),
        ("table", "HTMLTableElement"),
        ("tbody", "HTMLTableSectionElement"),
        ("td", "HTMLTableCellElement"),
        ("template", "HTMLTemplateElement"),
        ("textarea", "HTMLTextAreaElement"),
        ("tfoot", "HTMLTableSectionElement"),
        ("th", "HTMLTableCellElement"),
        ("thead", "HTMLTableSectionElement"),
        ("time", "HTMLTimeElement"),
        ("title", "HTMLTitleElement"),
        ("tr", "HTMLTableRowElement"),
        ("track", "HTMLTrackElement"),
        ("tt", "HTMLElement"),
        ("u", "HTMLElement"),
        ("ul", "HTMLUListElement"),
        ("var", "HTMLElement"),
        ("video", "HTMLVideoElement"),
        ("wbr", "HTMLElement"),
        ("xmp", "HTMLPreElement"),
    ];

    const SVG_EXPECTED: &[(&str, &str)] = &[
        ("a", "SVGAElement"),
        ("animate", "SVGAnimateElement"),
        ("animateMotion", "SVGAnimateMotionElement"),
        ("animateTransform", "SVGAnimateTransformElement"),
        ("circle", "SVGCircleElement"),
        ("clipPath", "SVGClipPathElement"),
        ("defs", "SVGDefsElement"),
        ("desc", "SVGDescElement"),
        ("ellipse", "SVGEllipseElement"),
        ("feBlend", "SVGFEBlendElement"),
        ("feColorMatrix", "SVGFEColorMatrixElement"),
        ("feComponentTransfer", "SVGFEComponentTransferElement"),
        ("feComposite", "SVGFECompositeElement"),
        ("feConvolveMatrix", "SVGFEConvolveMatrixElement"),
        ("feDiffuseLighting", "SVGFEDiffuseLightingElement"),
        ("feDisplacementMap", "SVGFEDisplacementMapElement"),
        ("feDistantLight", "SVGFEDistantLightElement"),
        ("feDropShadow", "SVGFEDropShadowElement"),
        ("feFlood", "SVGFEFloodElement"),
        ("feFuncA", "SVGFEFuncAElement"),
        ("feFuncB", "SVGFEFuncBElement"),
        ("feFuncG", "SVGFEFuncGElement"),
        ("feFuncR", "SVGFEFuncRElement"),
        ("feGaussianBlur", "SVGFEGaussianBlurElement"),
        ("feImage", "SVGFEImageElement"),
        ("feMerge", "SVGFEMergeElement"),
        ("feMergeNode", "SVGFEMergeNodeElement"),
        ("feMorphology", "SVGFEMorphologyElement"),
        ("feOffset", "SVGFEOffsetElement"),
        ("fePointLight", "SVGFEPointLightElement"),
        ("feSpecularLighting", "SVGFESpecularLightingElement"),
        ("feSpotLight", "SVGFESpotLightElement"),
        ("feTile", "SVGFETileElement"),
        ("feTurbulence", "SVGFETurbulenceElement"),
        ("filter", "SVGFilterElement"),
        ("foreignObject", "SVGForeignObjectElement"),
        ("g", "SVGGElement"),
        ("image", "SVGImageElement"),
        ("line", "SVGLineElement"),
        ("linearGradient", "SVGLinearGradientElement"),
        ("marker", "SVGMarkerElement"),
        ("mask", "SVGMaskElement"),
        ("metadata", "SVGMetadataElement"),
        ("mpath", "SVGMPathElement"),
        ("path", "SVGPathElement"),
        ("pattern", "SVGPatternElement"),
        ("polygon", "SVGPolygonElement"),
        ("polyline", "SVGPolylineElement"),
        ("radialGradient", "SVGRadialGradientElement"),
        ("rect", "SVGRectElement"),
        ("script", "SVGScriptElement"),
        ("set", "SVGSetElement"),
        ("stop", "SVGStopElement"),
        ("style", "SVGStyleElement"),
        ("svg", "SVGSVGElement"),
        ("switch", "SVGSwitchElement"),
        ("symbol", "SVGSymbolElement"),
        ("text", "SVGTextElement"),
        ("textPath", "SVGTextPathElement"),
        ("title", "SVGTitleElement"),
        ("tspan", "SVGTSpanElement"),
        ("use", "SVGUseElement"),
        ("view", "SVGViewElement"),
    ];

    #[test]
    fn inventory_is_unique_parent_first_and_all_lookup_rows_are_bound() {
        assert_eq!(INTERFACES.len(), 154);
        assert_eq!(INTERFACE_INDEX.len(), INTERFACES.len());
        for (index, descriptor) in INTERFACES.iter().enumerate() {
            assert_eq!(interface_index(descriptor.name), Some(index));
            assert_eq!(interface(descriptor.name), Some(descriptor));
            assert!(
                !INTERFACES[..index]
                    .iter()
                    .any(|row| row.name == descriptor.name)
            );
            if let Some(parent) = descriptor.parent {
                let parent_index = parent_index(index).expect("declared parent index");
                assert!(parent_index < index);
                assert_eq!(INTERFACES[parent_index].name, parent);
                assert!(INTERFACES[..index].iter().any(|row| row.name == parent));
            } else {
                assert_eq!(parent_index(index), None);
                assert_eq!(descriptor.name, "EventTarget");
            }
        }
        assert_eq!(parent_index(INTERFACES.len()), None);
        assert_eq!(parent_index(usize::MAX), None);
        assert!(interface_order().eq(INTERFACE_INDEX.iter().map(|row| row.1)));
        assert!(INTERFACE_INDEX.windows(2).all(|rows| rows[0].0 < rows[1].0));
        for &(name, index) in INTERFACE_INDEX {
            assert_eq!(INTERFACES[index].name, name);
        }
        for absent in [
            "HTMLDocument",
            "Attr",
            "CDATASection",
            "XMLDocument",
            "ShadowRoot",
        ] {
            assert_eq!(interface_index(absent), None, "{absent}");
            assert_eq!(interface(absent), None, "{absent}");
        }
    }

    #[test]
    fn constructor_categories_preserve_real_and_conditional_construction() {
        for (name, kind, length) in [
            ("Node", ConstructorKind::Illegal, 0),
            ("Element", ConstructorKind::Illegal, 0),
            ("CharacterData", ConstructorKind::Illegal, 0),
            ("DocumentType", ConstructorKind::Illegal, 0),
            ("Document", ConstructorKind::Document, 0),
            ("DocumentFragment", ConstructorKind::DocumentFragment, 0),
            ("Text", ConstructorKind::Text, 0),
            ("Comment", ConstructorKind::Comment, 0),
            (
                "ProcessingInstruction",
                ConstructorKind::ProcessingInstruction,
                1,
            ),
            ("EventTarget", ConstructorKind::EventTarget, 0),
            ("HTMLElement", ConstructorKind::Html, 0),
            ("HTMLDivElement", ConstructorKind::Html, 0),
            ("HTMLUnknownElement", ConstructorKind::Illegal, 0),
            ("HTMLMediaElement", ConstructorKind::Illegal, 0),
            ("SVGSVGElement", ConstructorKind::Illegal, 0),
            ("MathMLElement", ConstructorKind::Illegal, 0),
        ] {
            let descriptor = interface(name).unwrap();
            assert_eq!(
                (descriptor.constructor, descriptor.length),
                (kind, length),
                "{name}"
            );
        }
    }

    #[test]
    fn static_tag_tables_are_ordered_and_target_declared_interfaces() {
        assert_eq!(HTML_NAMES.len(), 149);
        assert_eq!(SVG_NAMES.len(), 63);
        for (table, expected) in [(HTML_NAMES, HTML_EXPECTED), (SVG_NAMES, SVG_EXPECTED)] {
            assert_eq!(table.len(), expected.len());
            assert!(table.windows(2).all(|rows| rows[0].0 < rows[1].0));
            for (&(name, target), &(expected_tag, expected_name)) in table.iter().zip(expected) {
                assert_eq!(name, expected_tag);
                assert_eq!(lookup(table, name), Some(target));
                assert_eq!(INTERFACES[target].name, expected_name, "{name}");
                assert!(
                    interface(expected_name).is_some(),
                    "{name}: {expected_name}"
                );
            }
        }
        for (index, expected) in [
            (DOCUMENT_INDEX, "Document"),
            (DOCUMENT_FRAGMENT_INDEX, "DocumentFragment"),
            (TEXT_INDEX, "Text"),
            (COMMENT_INDEX, "Comment"),
            (DOCUMENT_TYPE_INDEX, "DocumentType"),
            (PROCESSING_INSTRUCTION_INDEX, "ProcessingInstruction"),
            (HTML_ELEMENT_INDEX, "HTMLElement"),
            (HTML_UNKNOWN_ELEMENT_INDEX, "HTMLUnknownElement"),
            (SVG_ELEMENT_INDEX, "SVGElement"),
            (MATH_ML_ELEMENT_INDEX, "MathMLElement"),
        ] {
            assert_eq!(INTERFACES[index].name, expected);
        }
    }

    #[test]
    fn html_literal_families_keep_the_most_derived_interface() {
        for (tag, expected) in [
            ("section", "HTMLElement"),
            ("article", "HTMLElement"),
            ("search", "HTMLElement"),
            ("div", "HTMLDivElement"),
            ("a", "HTMLAnchorElement"),
            ("audio", "HTMLAudioElement"),
            ("video", "HTMLVideoElement"),
            ("h1", "HTMLHeadingElement"),
            ("h6", "HTMLHeadingElement"),
            ("q", "HTMLQuoteElement"),
            ("blockquote", "HTMLQuoteElement"),
            ("ins", "HTMLModElement"),
            ("del", "HTMLModElement"),
            ("col", "HTMLTableColElement"),
            ("colgroup", "HTMLTableColElement"),
            ("td", "HTMLTableCellElement"),
            ("th", "HTMLTableCellElement"),
            ("tbody", "HTMLTableSectionElement"),
            ("tfoot", "HTMLTableSectionElement"),
            ("thead", "HTMLTableSectionElement"),
            ("template", "HTMLTemplateElement"),
            ("slot", "HTMLSlotElement"),
            ("selectedcontent", "HTMLSelectedContentElement"),
        ] {
            assert_eq!(element_interface(Namespace::Html, tag), expected, "{tag}");
        }
        assert_eq!(
            interface("HTMLAudioElement").unwrap().parent,
            Some("HTMLMediaElement")
        );
        assert_eq!(
            interface("HTMLVideoElement").unwrap().parent,
            Some("HTMLMediaElement")
        );
        assert_eq!(
            interface("HTMLMediaElement").unwrap().parent,
            Some("HTMLElement")
        );
        assert_eq!(interface("HTMLElement").unwrap().parent, Some("Element"));
    }

    #[test]
    fn obsolete_html_rules_and_custom_reserved_names_are_not_conflated() {
        for tag in [
            "applet",
            "bgsound",
            "blink",
            "isindex",
            "keygen",
            "multicol",
            "nextid",
            "spacer",
            "annotation-xml",
            "color-profile",
            "font-face",
            "font-face-src",
            "font-face-uri",
            "font-face-format",
            "font-face-name",
            "missing-glyph",
        ] {
            assert_eq!(
                element_interface(Namespace::Html, tag),
                "HTMLUnknownElement",
                "{tag}"
            );
        }
        for tag in [
            "acronym",
            "basefont",
            "big",
            "center",
            "nobr",
            "noembed",
            "noframes",
            "plaintext",
            "rb",
            "rtc",
            "strike",
            "tt",
        ] {
            assert_eq!(
                element_interface(Namespace::Html, tag),
                "HTMLElement",
                "{tag}"
            );
        }
        for (tag, expected) in [
            ("listing", "HTMLPreElement"),
            ("xmp", "HTMLPreElement"),
            ("dir", "HTMLDirectoryElement"),
            ("font", "HTMLFontElement"),
            ("frame", "HTMLFrameElement"),
            ("frameset", "HTMLFrameSetElement"),
            ("marquee", "HTMLMarqueeElement"),
            ("param", "HTMLParamElement"),
        ] {
            assert_eq!(element_interface(Namespace::Html, tag), expected, "{tag}");
        }
    }

    #[test]
    fn current_custom_names_use_ascii_exclusions_not_old_pcen_ranges() {
        for name in [
            "x-",
            "x-widget",
            "math-α",
            "emotion-😍",
            "x-:",
            "x-!",
            "x-=",
            "x-\u{b}",
            "x-\u{80}",
            "x-\u{ffff}",
            "x-\u{10ffff}",
        ] {
            assert_eq!(
                element_interface(Namespace::Html, name),
                "HTMLElement",
                "{name:?}"
            );
        }
        for name in [
            "",
            "widget",
            "X-widget",
            "x-Widget",
            "-widget",
            "1-widget",
            "α-widget",
            "x-\0",
            "x-\t",
            "x-\n",
            "x-\u{c}",
            "x-\r",
            "x- ",
            "x-/",
            "x->",
        ] {
            assert_eq!(
                element_interface(Namespace::Html, name),
                "HTMLUnknownElement",
                "{name:?}"
            );
        }
    }

    #[test]
    fn svg_literal_chains_namespace_and_case_are_preserved() {
        for (tag, expected, parent) in [
            ("svg", "SVGSVGElement", "SVGGraphicsElement"),
            ("symbol", "SVGSymbolElement", "SVGGraphicsElement"),
            ("clipPath", "SVGClipPathElement", "SVGElement"),
            ("mask", "SVGMaskElement", "SVGElement"),
            ("path", "SVGPathElement", "SVGGeometryElement"),
            ("text", "SVGTextElement", "SVGTextPositioningElement"),
            ("tspan", "SVGTSpanElement", "SVGTextPositioningElement"),
            ("textPath", "SVGTextPathElement", "SVGTextContentElement"),
            (
                "linearGradient",
                "SVGLinearGradientElement",
                "SVGGradientElement",
            ),
            ("animate", "SVGAnimateElement", "SVGAnimationElement"),
            ("mpath", "SVGMPathElement", "SVGElement"),
            (
                "feFuncR",
                "SVGFEFuncRElement",
                "SVGComponentTransferFunctionElement",
            ),
            ("feDropShadow", "SVGFEDropShadowElement", "SVGElement"),
        ] {
            assert_eq!(element_interface(Namespace::Svg, tag), expected, "{tag}");
            assert_eq!(interface(expected).unwrap().parent, Some(parent));
        }
        for tag in ["div", "x-widget", "lineargradient", "unknown"] {
            assert_eq!(
                element_interface(Namespace::Svg, tag),
                "SVGElement",
                "{tag}"
            );
        }
        assert_eq!(
            element_interface(Namespace::Html, "DIV"),
            "HTMLUnknownElement"
        );
        for tag in ["math", "mrow", "mi", "annotation-xml", "svg", "unknown"] {
            assert_eq!(element_interface(Namespace::MathMl, tag), "MathMLElement");
        }
    }

    #[test]
    fn stored_parser_namespaces_and_template_fragments_drive_mapping() {
        let mut doc = Document::parse(
            "<!doctype html><body><svg id='s'><linearGradient id='g'></linearGradient><foreignObject><div id='h'></div></foreignObject></svg><math><mrow id='m'></mrow></math><template id='t'></template>",
        );
        for (selector, expected) in [
            ("#s", "SVGSVGElement"),
            ("#g", "SVGLinearGradientElement"),
            ("#h", "HTMLDivElement"),
            ("#m", "MathMLElement"),
            ("#t", "HTMLTemplateElement"),
        ] {
            assert_eq!(
                node_interface(&doc, doc.query_selector(selector).unwrap()),
                Some(expected)
            );
        }
        assert_eq!(node_interface(&doc, doc.root), Some("Document"));
        let template = doc.query_selector("#t").unwrap();
        assert_eq!(
            node_interface(&doc, doc.template_contents(template).unwrap()),
            Some("DocumentFragment")
        );
        let fragment = doc.create_document_fragment();
        let text = doc.create_text_node("x");
        let comment = doc.create_comment("x");
        let pi = doc.create_processing_instruction("target", "data");
        for (node, expected) in [
            (fragment, "DocumentFragment"),
            (text, "Text"),
            (comment, "Comment"),
            (pi, "ProcessingInstruction"),
        ] {
            assert_eq!(node_interface(&doc, node), Some(expected));
            assert_eq!(mapping_work(&doc, node), Some(4));
        }
        let doctype = doc
            .nodes
            .iter()
            .position(|node| matches!(node.kind, NodeKind::Doctype(_)))
            .unwrap();
        assert_eq!(node_interface(&doc, doctype), Some("DocumentType"));
        for id in 0..doc.nodes.len() {
            let index = node_interface_index(&doc, id).unwrap();
            assert!(index < INTERFACES.len());
            assert_eq!(node_interface(&doc, id), Some(INTERFACES[index].name));
        }
        assert_eq!(node_interface_index(&doc, doc.nodes.len()), None);
        assert_eq!(node_interface_index(&doc, usize::MAX), None);
        assert_eq!(node_interface(&doc, usize::MAX), None);
        assert_eq!(mapping_work(&doc, usize::MAX), None);
    }

    #[test]
    fn mapping_preflight_accounts_long_names_and_refuses_arithmetic_overflow() {
        let mut doc = Document::parse("");
        let tag = format!("x-{}", "a".repeat(8192));
        let bytes = tag.len();
        let id = doc.nodes.len();
        // Direct stored-node fixture avoids the separate parser/createElement
        // name clamp; it tests mapping's own length bound and no normalization.
        doc.nodes.push(Node {
            parent: None,
            children: Vec::new(),
            kind: NodeKind::Element(Element {
                namespace: Namespace::Html,
                tag,
                attrs: BTreeMap::new(),
                attr_namespaces: BTreeMap::new(),
                template_contents: None,
            }),
        });
        let paid = mapping_work(&doc, id).unwrap();
        assert!(paid >= bytes * 2);
        assert!(paid > mapping_work(&doc, doc.root).unwrap());
        assert_eq!(node_interface(&doc, id), Some("HTMLElement"));
        assert_eq!(element_mapping_work(Namespace::Html, usize::MAX), None);
        assert_eq!(element_mapping_work(Namespace::MathMl, usize::MAX), Some(4));
        // Table comparisons are bounded by their short static keys, so an
        // unbounded lookup key cannot overflow or force a full suffix scan.
        assert_eq!(
            interface_lookup_work(usize::MAX),
            interface_lookup_work(INTERFACE_MAX_NAME)
        );
        assert!(interface_lookup_work(0) > 0);
    }
}
