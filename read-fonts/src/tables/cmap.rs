//! The [cmap](https://docs.microsoft.com/en-us/typography/opentype/spec/cmap) table

use std::convert::TryInto;

use font_types::Tag;

/// 'cmap'
pub const TAG: Tag = Tag::new(b"cmap");

include!("../../generated/generated_cmap.rs");

impl Cmap4<'_> {
    // adapted from fonttools-rs
    /// An iterator over the char -> GlyphId mappings
    pub fn iter(&self) -> impl Iterator<Item = (char, GlyphId)> + '_ {
        struct RangeIter {
            next: u16,
            end: u16,
        }

        impl RangeIter {
            fn is_done(&self) -> bool {
                self.next > self.end
            }

            fn next(&mut self) -> Option<char> {
                if self.next <= self.end {
                    let next = self.next;
                    self.next += 1;
                    Some(char::from_u32(next as _).unwrap())
                } else {
                    None
                }
            }
        }

        let mut next_i = 0;
        let mut i = 0;
        let n = self.start_code().len().saturating_sub(1);

        let mut iter = RangeIter { next: 1, end: 0 };
        let mut start = 0;
        let mut delta = 0;
        let mut range_offset = 0;

        std::iter::from_fn(move || {
            if iter.is_done() {
                if next_i == n {
                    return None;
                }

                i = next_i;
                next_i += 1;
                start = self.start_code()[i].get();
                let end = self.end_code()[i].get();
                delta = self.id_delta()[i].get();
                range_offset = self.id_range_offsets()[i].get();
                if start == 0xffff {
                    return None;
                }
                iter = RangeIter { next: start, end };
            }

            let partial = (range_offset / 2) as i32 - (start as i32) + (i as i32)
                - (self.id_range_offsets().len() as i32);
            let char_code = iter.next()?;
            if range_offset == 0 {
                let gid = GlyphId::new((char_code as i32 + delta as i32) as _);
                Some((char_code, gid))
            } else {
                let index = (char_code as i32 + partial) as usize;
                // NOTE: I don't know when this happens, or if it happens?
                // should this be modular arithmetic or something? in the
                // fonttools-rs code, this breaks the inner loop
                assert!(index < self.glyph_id_array().len());
                if self.glyph_id_array()[index] != 0 {
                    let gid = self.glyph_id_array()[index].get() as i16 + delta;
                    let gid = GlyphId::new(gid.try_into().unwrap());

                    Some((char_code, gid))
                } else {
                    Some((char_code, GlyphId::NOTDEF))
                }
            }
        })
    }

    /// An iterator over GlyphId -> char mappings
    pub fn reversed(&self) -> impl Iterator<Item = (GlyphId, char)> + '_ {
        self.iter().map(|(chr, gid)| (gid, chr))
    }
}
