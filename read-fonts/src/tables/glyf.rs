//! The [glyf (Glyph Data)](https://docs.microsoft.com/en-us/typography/opentype/spec/glyf) table

use font_types::ReadScalar;

use crate::{FontData, ReadError};

/// 'glyf'
pub const TAG: Tag = Tag::new(b"glyf");

include!("../../generated/generated_glyf.rs");

macro_rules! field_getter {
    ($field:ident, $ty:ty) => {
        pub fn $field(&self) -> $ty {
            match self {
                Self::Simple(table) => table.$field(),
                Self::Composite(table) => table.$field(),
            }
        }
    };
}

impl<'a> Glyph<'a> {
    field_getter!(number_of_contours, i16);
    field_getter!(x_min, i16);
    field_getter!(x_max, i16);
    field_getter!(y_min, i16);
    field_getter!(y_max, i16);
}

//NOTE: This code below was taken from an old implementation, and has a bunch
// of funny warts. It should be replaced at some point, but might be useful in
// the interim?

impl<'a> SimpleGlyph<'a> {
    pub fn iter_points(&self) -> PointIter<'_> {
        self.iter_points_impl()
            .unwrap_or_else(|| PointIter::new(&[], &[], &[], &[]))
    }

    fn iter_points_impl(&self) -> Option<PointIter<'_>> {
        let end_points = self.end_pts_of_contours();
        let n_points = end_points.last()?.get().checked_add(1)?;
        let data = self.glyph_data();
        let lens = resolve_coords_len(data, n_points)?;
        let total_len = lens.flags + lens.x_coords + lens.y_coords;
        if data.len() < total_len as usize {
            return None;
        }

        let (flags, data) = data.split_at(lens.flags as usize);
        let (x_coords, y_coords) = data.split_at(lens.x_coords as usize);

        Some(PointIter::new(end_points, flags, x_coords, y_coords))
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Point {
    pub x: i16,
    pub y: i16,
}

#[derive(Clone, Copy, Debug)]
pub enum GlyphPoint {
    OffCurve(Point),
    OnCurve(Point),
    End(Point),
}

pub struct PointIter<'a> {
    end_points: &'a [BigEndian<u16>],
    cur_point: u16,
    flags: OldCursor<'a>,
    x_coords: OldCursor<'a>,
    y_coords: OldCursor<'a>,
    flag_repeats: u8,
    cur_flags: SimpleGlyphFlags,
    cur_x: i16,
    cur_y: i16,
}

impl<'a> Iterator for PointIter<'a> {
    type Item = GlyphPoint;
    fn next(&mut self) -> Option<GlyphPoint> {
        let next_end = self.end_points.first()?.get();
        let is_end = next_end <= self.cur_point; // LE because points could be out of order?
        if is_end {
            self.end_points = &self.end_points[1..];
        }
        self.advance_flags();
        self.advance_points();
        self.cur_point = self.cur_point.saturating_add(1);

        let point = Point {
            x: self.cur_x,
            y: self.cur_y,
        };

        if is_end {
            Some(GlyphPoint::End(point))
        } else if self.cur_flags.contains(SimpleGlyphFlags::ON_CURVE_POINT) {
            Some(GlyphPoint::OnCurve(point))
        } else {
            Some(GlyphPoint::OffCurve(point))
        }
    }
}

impl<'a> PointIter<'a> {
    fn new(
        end_points: &'a [BigEndian<u16>],
        flags: &'a [u8],
        x_coords: &'a [u8],
        y_coords: &'a [u8],
    ) -> Self {
        Self {
            end_points,
            flags: OldCursor::new(flags),
            x_coords: OldCursor::new(x_coords),
            y_coords: OldCursor::new(y_coords),
            cur_point: 0,
            flag_repeats: 0,
            cur_flags: SimpleGlyphFlags::empty(),
            cur_x: 0,
            cur_y: 0,
        }
    }

    fn advance_flags(&mut self) {
        if self.flag_repeats == 0 {
            self.cur_flags =
                SimpleGlyphFlags::from_bits_truncate(self.flags.bump().unwrap_or_default());
            self.flag_repeats = self
                .cur_flags
                .contains(SimpleGlyphFlags::REPEAT_FLAG)
                .then(|| self.flags.bump())
                .flatten()
                .unwrap_or(1);
        }
        self.flag_repeats -= 1;
    }

    fn advance_points(&mut self) {
        let x_short = self.cur_flags.contains(SimpleGlyphFlags::X_SHORT_VECTOR);
        let x_same_or_pos = self
            .cur_flags
            .contains(SimpleGlyphFlags::X_IS_SAME_OR_POSITIVE_X_SHORT_VECTOR);
        let y_short = self.cur_flags.contains(SimpleGlyphFlags::Y_SHORT_VECTOR);
        let y_same_or_pos = self
            .cur_flags
            .contains(SimpleGlyphFlags::Y_IS_SAME_OR_POSITIVE_Y_SHORT_VECTOR);

        let delta_x = match (x_short, x_same_or_pos) {
            (true, false) => -(self.x_coords.bump::<u8>().unwrap_or(0) as i16),
            (true, true) => self.x_coords.bump::<u8>().unwrap_or(0) as i16,
            (false, false) => self.x_coords.bump::<i16>().unwrap_or(0),
            _ => 0,
        };

        let delta_y = match (y_short, y_same_or_pos) {
            (true, false) => -(self.y_coords.bump::<u8>().unwrap_or(0) as i16),
            (true, true) => self.y_coords.bump::<u8>().unwrap_or(0) as i16,
            (false, false) => self.y_coords.bump::<i16>().unwrap_or(0),
            _ => 0,
        };

        self.cur_x = self.cur_x.wrapping_add(delta_x);
        self.cur_y = self.cur_y.wrapping_add(delta_y);
    }
}

//taken from ttf_parser https://docs.rs/ttf-parser/latest/src/ttf_parser/tables/glyf.rs.html#1-677
/// Resolves coordinate arrays length.
///
/// The length depends on *Simple Glyph Flags*, so we have to process them all to find it.
fn resolve_coords_len(data: &[u8], points_total: u16) -> Option<FieldLengths> {
    let mut cursor = OldCursor::new(data);

    let mut flags_left = u32::from(points_total);
    //let mut repeats;
    let mut x_coords_len = 0;
    let mut y_coords_len = 0;
    //let mut flags_seen = 0;
    while flags_left > 0 {
        let flags: SimpleGlyphFlags = cursor.bump()?;

        // The number of times a glyph point repeats.
        let repeats = if flags.contains(SimpleGlyphFlags::REPEAT_FLAG) {
            let repeats: u8 = cursor.bump()?;
            u32::from(repeats) + 1
        } else {
            1
        };

        if repeats > flags_left {
            return None;
        }

        // Non-obfuscated code below.
        // Branchless version is surprisingly faster.
        //
        // if flags.x_short() {
        //     // Coordinate is 1 byte long.
        //     x_coords_len += repeats;
        // } else if !flags.x_is_same_or_positive_short() {
        //     // Coordinate is 2 bytes long.
        //     x_coords_len += repeats * 2;
        // }
        // if flags.y_short() {
        //     // Coordinate is 1 byte long.
        //     y_coords_len += repeats;
        // } else if !flags.y_is_same_or_positive_short() {
        //     // Coordinate is 2 bytes long.
        //     y_coords_len += repeats * 2;
        // }
        let x_short = SimpleGlyphFlags::X_SHORT_VECTOR;
        let x_long = SimpleGlyphFlags::X_SHORT_VECTOR
            | SimpleGlyphFlags::X_IS_SAME_OR_POSITIVE_X_SHORT_VECTOR;
        let y_short = SimpleGlyphFlags::Y_SHORT_VECTOR;
        let y_long = SimpleGlyphFlags::Y_SHORT_VECTOR
            | SimpleGlyphFlags::Y_IS_SAME_OR_POSITIVE_Y_SHORT_VECTOR;
        x_coords_len += ((flags & x_short).bits() != 0) as u32 * repeats;
        x_coords_len += ((flags & x_long).bits() == 0) as u32 * repeats;

        y_coords_len += ((flags & y_short).bits() != 0) as u32 * repeats;
        y_coords_len += ((flags & y_long).bits() == 0) as u32 * repeats;

        flags_left -= repeats;
    }

    Some(FieldLengths {
        flags: cursor.pos as u32,
        x_coords: x_coords_len,
        y_coords: y_coords_len,
    })
    //Some((flags_len, x_coords_len, y_coords_len))
}

struct FieldLengths {
    flags: u32,
    x_coords: u32,
    y_coords: u32,
}

/// A slice of bytes and an index into them.
struct OldCursor<'a> {
    data: &'a [u8],
    pos: usize,
}

impl<'a> OldCursor<'a> {
    fn new(data: &'a [u8]) -> Self {
        Self { data, pos: 0 }
    }

    /// Attempt to read `T` at the current location, advancing if successful.
    fn bump<T: ReadScalar>(&mut self) -> Option<T> {
        let r = T::read(self.data.get(self.pos..)?);
        self.pos += T::RAW_BYTE_LEN;
        r
    }
}

/// A reference to another glyph. Part of [CompositeGlyph].
pub struct Component {
    pub flags: CompositeGlyphFlags,
    pub glyph: GlyphId,
    pub pos: ComponentPos,
    //TODO: transforms
}

#[derive(Debug, Clone, Copy)]
pub enum ComponentPos {
    Offset { x: i16, y: i16 },
    Point { base: u16, component: u16 },
}

impl<'a> CompositeGlyph<'a> {
    pub fn iter_components(&self) -> ComponentIter<'a> {
        ComponentIter {
            done: false,
            cursor: FontData::new(self.component_data()).cursor(),
        }
    }
}

pub struct ComponentIter<'a> {
    done: bool,
    cursor: Cursor<'a>,
}

impl Iterator for ComponentIter<'_> {
    type Item = Component;

    fn next(&mut self) -> Option<Self::Item> {
        if self.done {
            return None;
        }
        let flags: CompositeGlyphFlags = self.cursor.read().ok()?;
        let glyph = self.cursor.read::<GlyphId>().ok()?;
        let args_are_word = flags.contains(CompositeGlyphFlags::ARG_1_AND_2_ARE_WORDS);
        let are_signed = flags.contains(CompositeGlyphFlags::ARG_1_AND_2_ARE_WORDS);
        let pos = match (are_signed, args_are_word) {
            (true, true) => ComponentPos::Offset {
                x: self.cursor.read().ok()?,
                y: self.cursor.read().ok()?,
            },
            (true, false) => ComponentPos::Offset {
                x: self.cursor.read::<u8>().ok()? as _,
                y: self.cursor.read::<u8>().ok()? as _,
            },
            (false, true) => ComponentPos::Point {
                base: self.cursor.read().ok()?,
                component: self.cursor.read().ok()?,
            },
            (false, false) => ComponentPos::Point {
                base: self.cursor.read::<u8>().ok()? as _,
                component: self.cursor.read::<u8>().ok()? as _,
            },
        };

        let bytes_to_skip = if flags.contains(CompositeGlyphFlags::WE_HAVE_A_SCALE) {
            2
        } else if flags.contains(CompositeGlyphFlags::WE_HAVE_AN_X_AND_Y_SCALE) {
            4
        } else if flags.contains(CompositeGlyphFlags::WE_HAVE_A_TWO_BY_TWO) {
            8
        } else {
            0
        };
        self.cursor.advance_by(bytes_to_skip);
        self.done = !flags.contains(CompositeGlyphFlags::MORE_COMPONENTS);

        Some(Component { flags, glyph, pos })
    }
}

#[cfg(feature = "traversal")]
impl<'a> SomeTable<'a> for Component {
    fn type_name(&self) -> &str {
        "Component"
    }

    fn get_field(&self, idx: usize) -> Option<Field<'a>> {
        match idx {
            0 => Some(Field::new("flags", self.flags.bits())),
            1 => Some(Field::new("glyph", self.glyph)),
            2 => match self.pos {
                ComponentPos::Point { base, .. } => Some(Field::new("base", base)),
                ComponentPos::Offset { x, .. } => Some(Field::new("x", x)),
            },
            3 => match self.pos {
                ComponentPos::Point { component, .. } => Some(Field::new("component", component)),
                ComponentPos::Offset { y, .. } => Some(Field::new("y", y)),
            },
            _ => None,
        }
    }
}

//NOTE: we want generated_glyf traversal to include this:
//7usize => {
//let this = self.sneaky_copy();
//Some(Field::new(
//"components",
//FieldType::offset_iter(move || {
//Box::new(
//this.iter_components()
//.map(|item| FieldType::ResolvedOffset(Ok(Box::new(item)))),
//) as Box<dyn Iterator<Item = FieldType<'a>> + 'a>
//}),
//))
//}
