use parley::fontique::{Collection, CollectionOptions, SourceCache};
use parley::{FontContext, Layout, LayoutContext};
use vello::peniko::{Color, Fill};

pub mod cursor;
pub mod editor;

#[derive(Clone, PartialEq, Default, Debug)]
pub struct BrushIndex(pub usize);

pub struct ParleyLayoutWrapper(pub Layout<BrushIndex>);

pub struct TextContext {
    pub font_ctx: FontContext,
    pub layout_ctx: LayoutContext<BrushIndex>,
}

impl Default for TextContext {
    fn default() -> Self {
        Self::new()
    }
}

impl TextContext {
    pub fn new() -> Self {
        let font_ctx = FontContext {
            collection: Collection::new(CollectionOptions {
                system_fonts: true,
                ..Default::default()
            }),
            source_cache: SourceCache::default(),
        };
        Self { font_ctx, layout_ctx: LayoutContext::new() }
    }
}

pub fn render_text(
    layout: &Layout<BrushIndex>,
    scene: &mut vello::Scene,
    transform: vello::kurbo::Affine,
    _color: Color,
) {
    use parley::PositionedLayoutItem;

    let fill = Fill::NonZero;

    for line in layout.lines() {
        for item in line.items() {
            if let PositionedLayoutItem::GlyphRun(glyph_run) = item {
                let run = glyph_run.run();

                let vello_glyphs: Vec<vello::Glyph> =
                    glyph_run.glyphs().map(|g| vello::Glyph { id: g.id, x: g.x, y: g.y }).collect();

                scene
                    .draw_glyphs(run.font())
                    .font_size(run.font_size())
                    .transform(transform)
                    .draw(fill, vello_glyphs.into_iter());
            }
        }
    }
}
