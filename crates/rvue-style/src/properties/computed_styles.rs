//! Computed styles after cascade resolution.

use rudo_gc::{Trace, Visitor};

use crate::properties::{
    AlignItems, AlignSelf, BackgroundColor, BorderColor, BorderRadius, BorderStyle, BorderWidth,
    Color, Display, FlexDirection, FlexGrow, FlexShrink, FontFamily, FontSize, FontWeight, Gap,
    Height, JustifyContent, Margin, MaxHeight, MaxWidth, MinHeight, MinWidth, Opacity, Padding,
    TextColor, Visibility, Width, ZIndex,
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
    pub gap: Option<Gap>,
    pub border_color: Option<BorderColor>,
    pub border_width: Option<BorderWidth>,
    pub border_radius: Option<BorderRadius>,
    pub border_style: Option<BorderStyle>,
    pub opacity: Option<Opacity>,
    pub visibility: Option<Visibility>,
    pub z_index: Option<ZIndex>,
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
    }
}

unsafe impl Trace for ComputedStyles {
    fn trace(&self, _visitor: &mut impl Visitor) {}
}
