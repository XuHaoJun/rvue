use parley::{FontContext, Layout, LayoutContext};
use vello::peniko::{Color, Fill, Style};

#[derive(Clone, PartialEq, Default, Debug)]
pub struct BrushIndex(pub usize);

pub struct TextContext {
    pub font_ctx: FontContext,
    pub layout_ctx: LayoutContext<BrushIndex>,
}

impl TextContext {
    pub fn new() -> Self {
        Self { font_ctx: FontContext::new(), layout_ctx: LayoutContext::new() }
    }
}

pub fn render_text(
    layout: &Layout<BrushIndex>,
    scene: &mut vello::Scene,
    transform: vello::kurbo::Affine,
    color: Color,
) {
    use parley::PositionedLayoutItem;

    let fill = Fill::NonZero;

    for line in layout.lines() {
        for item in line.items() {
            if let PositionedLayoutItem::GlyphRun(glyph_run) = item {
                let run = glyph_run.run();

                let vello_glyphs: Vec<vello::Glyph> = glyph_run
                    .glyphs()
                    .map(|g| vello::Glyph { id: g.id, x: g.x as f32, y: g.y as f32 })
                    .collect();

                scene
                    .draw_glyphs(run.font())
                    .font_size(run.font_size())
                    .transform(transform)
                    .draw(fill, vello_glyphs.into_iter());
            }
        }
    }
}
