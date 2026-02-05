//! Computed styles after cascade resolution.

use rudo_gc::{Trace, Visitor};

use crate::properties::{
    AlignItems, AlignSelf, BackgroundColor, BorderColor, BorderRadius, BorderStyle, BorderWidth,
    Color, Cursor, Display, FlexBasis, FlexDirection, FlexGrow, FlexShrink, FontFamily, FontSize,
    FontWeight, Gap, Height, JustifyContent, Margin, MaxHeight, MaxWidth, MinHeight, MinWidth,
    Opacity, Overflow, Padding, TextColor, Visibility, Width, ZIndex,
};

#[derive(Default, Debug, Clone, PartialEq)]
pub struct ComputedStyles {
    pub background_color: Option<BackgroundColor>,
    pub color: Option<Color>,
    pub text_color: Option<TextColor>,
    pub font_size: Option<FontSize>,
    pub font_family: Option<FontFamily>,
    pub font_weight: Option<FontWeight>,
    pub padding: Option<Padding>,
    pub margin: Option<Margin>,
    pub width: Option<Width>,
    pub height: Option<Height>,
    pub min_width: Option<MinWidth>,
    pub min_height: Option<MinHeight>,
    pub max_width: Option<MaxWidth>,
    pub max_height: Option<MaxHeight>,
    pub display: Option<Display>,
    pub flex_direction: Option<FlexDirection>,
    pub justify_content: Option<JustifyContent>,
    pub align_items: Option<AlignItems>,
    pub align_self: Option<AlignSelf>,
    pub flex_grow: Option<FlexGrow>,
    pub flex_shrink: Option<FlexShrink>,
    pub flex_basis: Option<FlexBasis>,
    pub gap: Option<Gap>,
    pub border_color: Option<BorderColor>,
    pub border_width: Option<BorderWidth>,
    pub border_radius: Option<BorderRadius>,
    pub border_style: Option<BorderStyle>,
    pub opacity: Option<Opacity>,
    pub visibility: Option<Visibility>,
    pub z_index: Option<ZIndex>,
    pub cursor: Option<Cursor>,
    pub overflow_x: Option<Overflow>,
    pub overflow_y: Option<Overflow>,
}

impl ComputedStyles {
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    #[inline]
    pub fn merge(&mut self, properties: &crate::Properties) {
        if let Some(bg) = properties.get::<BackgroundColor>() {
            self.background_color = Some(*bg);
        }
        if let Some(c) = properties.get::<Color>() {
            self.color = Some(*c);
        }
        if let Some(c) = properties.get::<TextColor>() {
            self.text_color = Some(*c);
        }
        if let Some(fs) = properties.get::<FontSize>() {
            self.font_size = Some(*fs);
        }
        if let Some(ff) = properties.get::<FontFamily>() {
            self.font_family = Some(ff.clone());
        }
        if let Some(fw) = properties.get::<FontWeight>() {
            self.font_weight = Some(*fw);
        }
        if let Some(p) = properties.get::<Padding>() {
            self.padding = Some(*p);
        }
        if let Some(m) = properties.get::<Margin>() {
            self.margin = Some(*m);
        }
        if let Some(w) = properties.get::<Width>() {
            self.width = Some(w.clone());
        }
        if let Some(h) = properties.get::<Height>() {
            self.height = Some(h.clone());
        }
        if let Some(mw) = properties.get::<MinWidth>() {
            self.min_width = Some(mw.clone());
        }
        if let Some(mh) = properties.get::<MinHeight>() {
            self.min_height = Some(mh.clone());
        }
        if let Some(mw) = properties.get::<MaxWidth>() {
            self.max_width = Some(mw.clone());
        }
        if let Some(mh) = properties.get::<MaxHeight>() {
            self.max_height = Some(mh.clone());
        }
        if let Some(d) = properties.get::<Display>() {
            self.display = Some(*d);
        }
        if let Some(fd) = properties.get::<FlexDirection>() {
            self.flex_direction = Some(*fd);
        }
        if let Some(jc) = properties.get::<JustifyContent>() {
            self.justify_content = Some(*jc);
        }
        if let Some(ai) = properties.get::<AlignItems>() {
            self.align_items = Some(*ai);
        }
        if let Some(as_) = properties.get::<AlignSelf>() {
            self.align_self = Some(*as_);
        }
        if let Some(fg) = properties.get::<FlexGrow>() {
            self.flex_grow = Some(*fg);
        }
        if let Some(fs) = properties.get::<FlexShrink>() {
            self.flex_shrink = Some(*fs);
        }
        if let Some(fb) = properties.get::<FlexBasis>() {
            self.flex_basis = Some(fb.clone());
        }
        if let Some(g) = properties.get::<Gap>() {
            self.gap = Some(*g);
        }
        if let Some(bc) = properties.get::<BorderColor>() {
            self.border_color = Some(*bc);
        }
        if let Some(bw) = properties.get::<BorderWidth>() {
            self.border_width = Some(*bw);
        }
        if let Some(br) = properties.get::<BorderRadius>() {
            self.border_radius = Some(*br);
        }
        if let Some(bs) = properties.get::<BorderStyle>() {
            self.border_style = Some(*bs);
        }
        if let Some(o) = properties.get::<Opacity>() {
            self.opacity = Some(*o);
        }
        if let Some(v) = properties.get::<Visibility>() {
            self.visibility = Some(*v);
        }
        if let Some(zi) = properties.get::<ZIndex>() {
            self.z_index = Some(*zi);
        }
        if let Some(c) = properties.get::<Cursor>() {
            self.cursor = Some(c.clone());
        }
        if let Some(ox) = properties.get::<Overflow>() {
            self.overflow_x = Some(*ox);
        }
        if let Some(oy) = properties.get::<Overflow>() {
            self.overflow_y = Some(*oy);
        }
    }

    #[inline]
    pub fn merge_with_computed(&mut self, other: &ComputedStyles) {
        if let Some(bg) = other.background_color.as_ref() {
            self.background_color = Some(bg.clone());
        }
        if let Some(c) = other.color.as_ref() {
            self.color = Some(c.clone());
        }
        if let Some(tc) = other.text_color.as_ref() {
            self.text_color = Some(tc.clone());
        }
        if let Some(fs) = other.font_size.as_ref() {
            self.font_size = Some(fs.clone());
        }
        if let Some(ff) = other.font_family.as_ref() {
            self.font_family = Some(ff.clone());
        }
        if let Some(fw) = other.font_weight.as_ref() {
            self.font_weight = Some(fw.clone());
        }
        if let Some(p) = other.padding.as_ref() {
            self.padding = Some(p.clone());
        }
        if let Some(m) = other.margin.as_ref() {
            self.margin = Some(m.clone());
        }
        if let Some(w) = other.width.as_ref() {
            self.width = Some(w.clone());
        }
        if let Some(h) = other.height.as_ref() {
            self.height = Some(h.clone());
        }
        if let Some(mw) = other.min_width.as_ref() {
            self.min_width = Some(mw.clone());
        }
        if let Some(mh) = other.min_height.as_ref() {
            self.min_height = Some(mh.clone());
        }
        if let Some(mw) = other.max_width.as_ref() {
            self.max_width = Some(mw.clone());
        }
        if let Some(mh) = other.max_height.as_ref() {
            self.max_height = Some(mh.clone());
        }
        if let Some(d) = other.display.as_ref() {
            self.display = Some(d.clone());
        }
        if let Some(fd) = other.flex_direction.as_ref() {
            self.flex_direction = Some(fd.clone());
        }
        if let Some(jc) = other.justify_content.as_ref() {
            self.justify_content = Some(jc.clone());
        }
        if let Some(ai) = other.align_items.as_ref() {
            self.align_items = Some(ai.clone());
        }
        if let Some(as_) = other.align_self.as_ref() {
            self.align_self = Some(as_.clone());
        }
        if let Some(fg) = other.flex_grow.as_ref() {
            self.flex_grow = Some(fg.clone());
        }
        if let Some(fs) = other.flex_shrink.as_ref() {
            self.flex_shrink = Some(fs.clone());
        }
        if let Some(fb) = other.flex_basis.as_ref() {
            self.flex_basis = Some(fb.clone());
        }
        if let Some(g) = other.gap.as_ref() {
            self.gap = Some(g.clone());
        }
        if let Some(bc) = other.border_color.as_ref() {
            self.border_color = Some(bc.clone());
        }
        if let Some(bw) = other.border_width.as_ref() {
            self.border_width = Some(bw.clone());
        }
        if let Some(br) = other.border_radius.as_ref() {
            self.border_radius = Some(br.clone());
        }
        if let Some(bs) = other.border_style.as_ref() {
            self.border_style = Some(bs.clone());
        }
        if let Some(o) = other.opacity.as_ref() {
            self.opacity = Some(o.clone());
        }
        if let Some(v) = other.visibility.as_ref() {
            self.visibility = Some(v.clone());
        }
        if let Some(zi) = other.z_index.as_ref() {
            self.z_index = Some(zi.clone());
        }
        if let Some(c) = other.cursor.as_ref() {
            self.cursor = Some(c.clone());
        }
        if let Some(ox) = other.overflow_x.as_ref() {
            self.overflow_x = Some(*ox);
        }
        if let Some(oy) = other.overflow_y.as_ref() {
            self.overflow_y = Some(*oy);
        }
    }
}

unsafe impl Trace for ComputedStyles {
    fn trace(&self, visitor: &mut impl Visitor) {
        self.background_color.trace(visitor);
        self.color.trace(visitor);
        self.text_color.trace(visitor);
        self.font_size.trace(visitor);
        self.font_family.trace(visitor);
        self.font_weight.trace(visitor);
        self.padding.trace(visitor);
        self.margin.trace(visitor);
        self.width.trace(visitor);
        self.height.trace(visitor);
        self.min_width.trace(visitor);
        self.min_height.trace(visitor);
        self.max_width.trace(visitor);
        self.max_height.trace(visitor);
        self.display.trace(visitor);
        self.flex_direction.trace(visitor);
        self.justify_content.trace(visitor);
        self.align_items.trace(visitor);
        self.align_self.trace(visitor);
        self.flex_grow.trace(visitor);
        self.flex_shrink.trace(visitor);
        self.flex_basis.trace(visitor);
        self.gap.trace(visitor);
        self.border_color.trace(visitor);
        self.border_width.trace(visitor);
        self.border_radius.trace(visitor);
        self.border_style.trace(visitor);
        self.opacity.trace(visitor);
        self.visibility.trace(visitor);
        self.z_index.trace(visitor);
        self.cursor.trace(visitor);
        self.overflow_x.trace(visitor);
        self.overflow_y.trace(visitor);
    }
}
