//! Literal page pixels for authored background-layer order and geometry.
use eris::{
    graphics::{Canvas, Color, Fonts},
    page::Page,
};
use url::Url;

#[cfg(feature = "raster-bridge")]
#[allow(dead_code)]
#[path = "support/vulkan_backgrounds.rs"]
mod witness;

const WIDTH: u32 = 12;
const HEIGHT: u32 = 8;

fn check(style: &str, rows: &[&str; 8]) {
    let document = format!(
        "<!doctype html><style>html,body{{margin:0;background:white}}\
         #board{{position:absolute;left:2px;top:2px;width:6px;height:2px;\
         background-repeat:no-repeat;{style}}}</style><div id=board></div>"
    );
    let page = Page::from_html(
        Url::parse("https://layers.invalid/").unwrap(),
        &document,
        false,
    );
    let fonts = Fonts::new();
    let layout = page.layout(WIDTH as f32, HEIGHT as f32, &fonts);
    let expected: Vec<u32> = rows
        .iter()
        .flat_map(|row| {
            assert_eq!(row.len(), WIDTH as usize);
            row.bytes().map(|byte| match byte {
                b'W' => 0xffffff,
                b'R' => 0xff0000,
                b'G' => 0x00ff00,
                b'B' => 0x0000ff,
                _ => panic!("unknown literal pixel"),
            })
        })
        .collect();
    let mut canvas = Canvas::new(WIDTH, HEIGHT).unwrap();
    canvas.clear(Color::WHITE);
    canvas.paint(&layout.commands, &fonts, &page.images, 0.0, 0.0);
    assert!(!canvas.exhausted());
    assert_eq!(canvas.pixels, expected, "CSS: {style}");
    #[cfg(feature = "raster-bridge")]
    {
        use eris::graphics::raster_bridge::native::{NativePhase, plan_native_scene};
        let frame = eris_raster_core::Frame::new(WIDTH, HEIGHT, 0xffffff);
        let native = plan_native_scene(
            frame,
            &[NativePhase {
                frame,
                commands: &layout.commands,
            }],
            &page.images,
            &fonts,
        )
        .expect("opaque square layers admit Native");
        assert_eq!(
            witness::packed_pixels(native.plan()),
            expected,
            "Native CSS: {style}"
        );
    }
}

#[test]
fn first_image_is_topmost_and_none_preserves_geometry_slot() {
    check(
        "background-color:#0f0;background-image:linear-gradient(red,red),none,linear-gradient(blue,blue);background-size:2px 2px;background-position:0 0,2px 0,4px 0",
        &[
            "WWWWWWWWWWWW",
            "WWWWWWWWWWWW",
            "WWRRGGBBWWWW",
            "WWRRGGBBWWWW",
            "WWWWWWWWWWWW",
            "WWWWWWWWWWWW",
            "WWWWWWWWWWWW",
            "WWWWWWWWWWWW",
        ],
    );
    check(
        "background-image:linear-gradient(red,red),linear-gradient(blue,blue);background-size:2px 2px,6px 2px",
        &[
            "WWWWWWWWWWWW",
            "WWWWWWWWWWWW",
            "WWRRBBBBWWWW",
            "WWRRBBBBWWWW",
            "WWWWWWWWWWWW",
            "WWWWWWWWWWWW",
            "WWWWWWWWWWWW",
            "WWWWWWWWWWWW",
        ],
    );
    check(
        "background-image:linear-gradient(blue,blue),linear-gradient(red,red);background-size:6px 2px,2px 2px",
        &[
            "WWWWWWWWWWWW",
            "WWWWWWWWWWWW",
            "WWBBBBBBWWWW",
            "WWBBBBBBWWWW",
            "WWWWWWWWWWWW",
            "WWWWWWWWWWWW",
            "WWWWWWWWWWWW",
            "WWWWWWWWWWWW",
        ],
    );
}

#[test]
fn shorter_position_list_cycles_using_authored_image_indices() {
    check(
        "background-image:none,linear-gradient(#0f0,#0f0),linear-gradient(blue,blue);background-size:2px 2px;background-position:0 0,4px 0",
        &[
            "WWWWWWWWWWWW",
            "WWWWWWWWWWWW",
            "WWBBWWGGWWWW",
            "WWBBWWGGWWWW",
            "WWWWWWWWWWWW",
            "WWWWWWWWWWWW",
            "WWWWWWWWWWWW",
            "WWWWWWWWWWWW",
        ],
    );
}

#[test]
fn color_uses_bottom_layer_clip_even_when_every_image_is_none() {
    check(
        "left:1px;top:1px;padding:1px;border:1px solid transparent;background-color:#0f0;background-image:none,none;background-clip:border-box,content-box",
        &[
            "WWWWWWWWWWWW",
            "WWWWWWWWWWWW",
            "WWWWWWWWWWWW",
            "WWWGGGGGGWWW",
            "WWWGGGGGGWWW",
            "WWWWWWWWWWWW",
            "WWWWWWWWWWWW",
            "WWWWWWWWWWWW",
        ],
    );
}

#[test]
fn shorthand_preserves_top_layer_and_bottom_color_geometry() {
    check(
        "background:linear-gradient(red,red) 0 0/2px 2px no-repeat,linear-gradient(blue,blue) 4px 0/2px 2px no-repeat #0f0",
        &[
            "WWWWWWWWWWWW",
            "WWWWWWWWWWWW",
            "WWRRGGBBWWWW",
            "WWRRGGBBWWWW",
            "WWWWWWWWWWWW",
            "WWWWWWWWWWWW",
            "WWWWWWWWWWWW",
            "WWWWWWWWWWWW",
        ],
    );
}
