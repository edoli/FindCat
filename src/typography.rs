//! Perceptual glyph coverage for UI rendered into Bevy's linear-light framebuffer.
//! Keep dark-on-paper and light-on-green atlases separate: their coverage curves differ.
use crate::*;
use bevy::{asset::AssetId, text::TextLayoutInfo};
use std::collections::HashSet;

#[derive(Resource)]
pub struct Coverage {
    processed: HashSet<(AssetId<Image>, usize)>,
    dark: [u8; 256],
    light: [u8; 256],
}
impl Default for Coverage {
    fn default() -> Self {
        Self {
            processed: HashSet::new(),
            dark: coverage_lut([0.09, 0.17, 0.15], [0.99, 0.975, 0.94]),
            light: coverage_lut([0.99, 0.975, 0.94], [0.15, 0.30, 0.25]),
        }
    }
}
fn luminance(rgb: [f32; 3]) -> f32 {
    let linear = rgb.map(|s| {
        if s <= 0.04045 {
            s / 12.92
        } else {
            ((s + 0.055) / 1.055).powf(2.4)
        }
    });
    linear[0] * 0.2126 + linear[1] * 0.7152 + linear[2] * 0.0722
}
fn coverage_lut(fg: [f32; 3], bg: [f32; 3]) -> [u8; 256] {
    let f = luminance(fg);
    let b = luminance(bg);
    std::array::from_fn(|i| {
        let alpha = i as f32 / 255.;
        let target = luminance(std::array::from_fn(|c| bg[c] + (fg[c] - bg[c]) * alpha));
        (((target - b) / (f - b)).clamp(0., 1.) * 255.).round() as u8
    })
}

pub fn prepare_light_font(media: Res<Media>, mut fonts: ResMut<Assets<Font>>) {
    if fonts.get(&media.light_bold).is_none() {
        if let Some(font) = fonts.get(&media.bold).cloned() {
            // Font bytes share an Arc; only the glyph coverage atlases are independent.
            let _ = fonts.insert(media.light_bold.id(), font);
        }
    }
}

pub fn correct_coverage(
    text: Query<(&TextFont, &TextLayoutInfo), Changed<TextLayoutInfo>>,
    media: Res<Media>,
    layouts: Res<Assets<TextureAtlasLayout>>,
    mut images: ResMut<Assets<Image>>,
    mut coverage: ResMut<Coverage>,
) {
    for (font, layout) in &text {
        let lut = if font.font == media.light_bold {
            coverage.light
        } else {
            coverage.dark
        };
        for glyph in &layout.glyphs {
            let info = &glyph.atlas_info;
            let key = (info.texture, info.location.glyph_index);
            if coverage.processed.contains(&key) {
                continue;
            }
            let Some(atlas) = layouts.get(info.texture_atlas) else {
                continue;
            };
            let rect = atlas.textures[info.location.glyph_index];
            let Some(image) = images.get_mut(info.texture) else {
                continue;
            };
            let stride = image.width() as usize;
            let Some(data) = image.data.as_mut() else {
                continue;
            };
            for y in rect.min.y..rect.max.y {
                for x in rect.min.x..rect.max.x {
                    let index = ((y as usize) * stride + x as usize) * 4 + 3;
                    data[index] = lut[data[index] as usize];
                }
            }
            coverage.processed.insert(key);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn glyph_coverage_matches_perceptual_blending_in_both_polarities() {
        for (fg, bg) in [
            ([0.09, 0.17, 0.15], [0.99, 0.975, 0.94]),
            ([0.99, 0.975, 0.94], [0.15, 0.30, 0.25]),
        ] {
            let lut = coverage_lut(fg, bg);
            assert_eq!((lut[0], lut[255]), (0, 255));
            assert!(lut.windows(2).all(|p| p[0] <= p[1]));
            for sample in [32, 64, 128, 192, 224] {
                let alpha = sample as f32 / 255.;
                let expected = luminance(std::array::from_fn(|c| bg[c] + (fg[c] - bg[c]) * alpha));
                let actual =
                    luminance(bg) + (luminance(fg) - luminance(bg)) * lut[sample] as f32 / 255.;
                assert!((actual - expected).abs() < 0.003);
            }
        }
    }
}
