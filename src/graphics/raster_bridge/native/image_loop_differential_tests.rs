use super::*;

#[test]
fn native_image_reservation_matches_the_actual_canvas_loop_debit() {
    use crate::graphics::PaintBudget;
    use std::sync::Arc;

    let image = Arc::new(RasterImage {
        width: 1,
        height: 1,
        rgba: vec![23, 47, 89, 127],
    });
    let mut images = ImageStore::new();
    images.insert("pixel".into(), image.clone());
    let fonts = Fonts::new();
    let target = Frame::new(13, 9, 0xffffff);
    let clips = [
        Rect::new(0.0, 0.0, 13.0, 9.0),
        Rect::new(0.25, 0.75, 7.5, 4.5),
        Rect::new(-2.0, -1.25, 3.0, 3.75),
        Rect::new(11.75, 7.25, 4.5, 3.5),
    ];
    let positions_x = [-2.0, f32::from_bits(0xbfffec57), -0.25, 0.5, 11.75, 15.0];
    let positions_y = [-2.5, -0.25, 0.75, 7.5, 12.0];
    let sizes = [-0.5, 0.0, 0.25, 2.75, 15.5];
    let mut checked = 0;
    for clip in clips {
        for x in positions_x {
            for y in positions_y {
                for width in sizes {
                    for height in sizes {
                        let rect = BrowserRect {
                            x,
                            y,
                            width,
                            height,
                        };
                        let command = DrawCommand::Image {
                            rect,
                            key: "pixel".into(),
                        };
                        let mut frame = target;
                        frame.caller_clip = clip;
                        let prepared = plan_native_scene(
                            target,
                            &[NativePhase {
                                frame,
                                commands: &[command],
                            }],
                            &images,
                            &fonts,
                        )
                        .unwrap();

                        // Observe the software painter's actual debit. This
                        // oracle performs no copy of the reservation formula.
                        let mut canvas = Canvas::new(target.width, target.height).unwrap();
                        canvas.set_clip(BrowserRect {
                            x: clip.x,
                            y: clip.y,
                            width: clip.width,
                            height: clip.height,
                        });
                        let allowance = 1_000_000;
                        canvas.budget = Some(PaintBudget {
                            pixels: allowance,
                            glyphs: 0,
                        });
                        canvas.image(rect, &image);
                        assert!(!canvas.exhausted());
                        let debit = allowance - canvas.budget.as_ref().unwrap().pixels;
                        assert_eq!(
                            prepared.stats().cpu_pixel_upper_bound,
                            debit,
                            "rect={rect:?} clip={clip:?}"
                        );
                        checked += 1;
                    }
                }
            }
        }
    }
    assert_eq!(checked, 3000);
}
