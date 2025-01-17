// THIS FILE IS AUTOGENERATED.
// Any changes to this file will be overwritten.
// For more information about how codegen works, see font-codegen/README.md

#[allow(unused_imports)]
use font_types::*;

/// The [glyf (Glyph Data)](https://docs.microsoft.com/en-us/typography/opentype/spec/glyf) table
pub struct Glyf<'a> {
    offset_bytes: &'a [u8],
}

impl<'a> font_types::FontRead<'a> for Glyf<'a> {
    fn read(bytes: &'a [u8]) -> Option<Self> {
        let offset_bytes = bytes;
        let _bytes = bytes;
        let result = Glyf { offset_bytes };
        Some(result)
    }
}

impl<'a> Glyf<'a> {}

impl<'a> font_types::OffsetHost<'a> for Glyf<'a> {
    fn bytes(&self) -> &'a [u8] {
        self.offset_bytes
    }
}

/// The [Glyph Header](https://docs.microsoft.com/en-us/typography/opentype/spec/glyf#glyph-headers)
#[derive(Clone, Copy, Debug, zerocopy :: FromBytes, zerocopy :: Unaligned)]
#[repr(C)]
pub struct GlyphHeader {
    /// If the number of contours is greater than or equal to zero,
    /// this is a simple glyph. If negative, this is a composite glyph
    /// — the value -1 should be used for composite glyphs.
    pub number_of_contours: BigEndian<i16>,
    /// Minimum x for coordinate data.
    pub x_min: BigEndian<i16>,
    /// Minimum y for coordinate data.
    pub y_min: BigEndian<i16>,
    /// Maximum x for coordinate data.
    pub x_max: BigEndian<i16>,
    /// Maximum y for coordinate data.
    pub y_max: BigEndian<i16>,
}

impl GlyphHeader {
    /// If the number of contours is greater than or equal to zero,
    /// this is a simple glyph. If negative, this is a composite glyph
    /// — the value -1 should be used for composite glyphs.
    pub fn number_of_contours(&self) -> i16 {
        self.number_of_contours.get()
    }

    /// Minimum x for coordinate data.
    pub fn x_min(&self) -> i16 {
        self.x_min.get()
    }

    /// Minimum y for coordinate data.
    pub fn y_min(&self) -> i16 {
        self.y_min.get()
    }

    /// Maximum x for coordinate data.
    pub fn x_max(&self) -> i16 {
        self.x_max.get()
    }

    /// Maximum y for coordinate data.
    pub fn y_max(&self) -> i16 {
        self.y_max.get()
    }
}

/// The [Glyph Header](https://docs.microsoft.com/en-us/typography/opentype/spec/glyf#glyph-headers)
pub struct SimpleGlyph<'a> {
    header: zerocopy::LayoutVerified<&'a [u8], GlyphHeader>,
    end_pts_of_contours: zerocopy::LayoutVerified<&'a [u8], [BigEndian<u16>]>,
    instruction_length: zerocopy::LayoutVerified<&'a [u8], BigEndian<u16>>,
    instructions: zerocopy::LayoutVerified<&'a [u8], [BigEndian<u8>]>,
    glyph_data: zerocopy::LayoutVerified<&'a [u8], [u8]>,
}

impl<'a> font_types::FontRead<'a> for SimpleGlyph<'a> {
    fn read(bytes: &'a [u8]) -> Option<Self> {
        let (header, bytes) =
            zerocopy::LayoutVerified::<_, GlyphHeader>::new_unaligned_from_prefix(bytes)?;
        let __resolved_header = &header;
        let (end_pts_of_contours, bytes) =
            zerocopy::LayoutVerified::<_, [BigEndian<u16>]>::new_slice_unaligned_from_prefix(
                bytes,
                get_n_contours(__resolved_header) as usize,
            )?;
        let (instruction_length, bytes) =
            zerocopy::LayoutVerified::<_, BigEndian<u16>>::new_unaligned_from_prefix(bytes)?;
        let __resolved_instruction_length = instruction_length.get();
        let (instructions, bytes) =
            zerocopy::LayoutVerified::<_, [BigEndian<u8>]>::new_slice_unaligned_from_prefix(
                bytes,
                __resolved_instruction_length as usize,
            )?;
        let (glyph_data, bytes) = (
            zerocopy::LayoutVerified::<_, [u8]>::new_slice_unaligned(bytes)?,
            0,
        );
        let _bytes = bytes;
        let result = SimpleGlyph {
            header,
            end_pts_of_contours,
            instruction_length,
            instructions,
            glyph_data,
        };
        Some(result)
    }
}

impl<'a> SimpleGlyph<'a> {
    pub fn header(&self) -> &GlyphHeader {
        &self.header
    }

    pub fn end_pts_of_contours(&self) -> &[BigEndian<u16>] {
        &self.end_pts_of_contours
    }

    /// Total number of bytes for instructions. If instructionLength is
    /// zero, no instructions are present for this glyph, and this
    /// field is followed directly by the flags field.
    pub fn instruction_length(&self) -> u16 {
        self.instruction_length.get()
    }

    /// Array of instruction byte code for the glyph.
    pub fn instructions(&self) -> &[BigEndian<u8>] {
        &self.instructions
    }

    /// the raw data for flags & x/y coordinates
    pub fn glyph_data(&self) -> &[u8] {
        &self.glyph_data
    }
}

bitflags::bitflags! { # [doc = " Flags used in [SimpleGlyph]"] pub struct SimpleGlyphFlags : u8 { # [doc = " Bit 0: If set, the point is on the curve; otherwise, it is off"] # [doc = " the curve."] const ON_CURVE_POINT = 0x01 ; # [doc = " Bit 1: If set, the corresponding x-coordinate is 1 byte long,"] # [doc = " and the sign is determined by the"] # [doc = " X_IS_SAME_OR_POSITIVE_X_SHORT_VECTOR flag. If not set, its"] # [doc = " interpretation depends on the"] # [doc = " X_IS_SAME_OR_POSITIVE_X_SHORT_VECTOR flag: If that other flag"] # [doc = " is set, the x-coordinate is the same as the previous"] # [doc = " x-coordinate, and no element is added to the xCoordinates"] # [doc = " array. If both flags are not set, the corresponding element in"] # [doc = " the xCoordinates array is two bytes and interpreted as a signed"] # [doc = " integer. See the description of the"] # [doc = " X_IS_SAME_OR_POSITIVE_X_SHORT_VECTOR flag for additional"] # [doc = " information."] const X_SHORT_VECTOR = 0x02 ; # [doc = " Bit 2: If set, the corresponding y-coordinate is 1 byte long,"] # [doc = " and the sign is determined by the"] # [doc = " Y_IS_SAME_OR_POSITIVE_Y_SHORT_VECTOR flag. If not set, its"] # [doc = " interpretation depends on the"] # [doc = " Y_IS_SAME_OR_POSITIVE_Y_SHORT_VECTOR flag: If that other flag"] # [doc = " is set, the y-coordinate is the same as the previous"] # [doc = " y-coordinate, and no element is added to the yCoordinates"] # [doc = " array. If both flags are not set, the corresponding element in"] # [doc = " the yCoordinates array is two bytes and interpreted as a signed"] # [doc = " integer. See the description of the"] # [doc = " Y_IS_SAME_OR_POSITIVE_Y_SHORT_VECTOR flag for additional"] # [doc = " information."] const Y_SHORT_VECTOR = 0x04 ; # [doc = " Bit 3: If set, the next byte (read as unsigned) specifies the"] # [doc = " number of additional times this flag byte is to be repeated in"] # [doc = " the logical flags array — that is, the number of additional"] # [doc = " logical flag entries inserted after this entry. (In the"] # [doc = " expanded logical array, this bit is ignored.) In this way, the"] # [doc = " number of flags listed can be smaller than the number of points"] # [doc = " in the glyph description."] const REPEAT_FLAG = 0x08 ; # [doc = " Bit 4: This flag has two meanings, depending on how the"] # [doc = " X_SHORT_VECTOR flag is set. If X_SHORT_VECTOR is set, this bit"] # [doc = " describes the sign of the value, with 1 equalling positive and"] # [doc = " 0 negative. If X_SHORT_VECTOR is not set and this bit is set,"] # [doc = " then the current x-coordinate is the same as the previous"] # [doc = " x-coordinate. If X_SHORT_VECTOR is not set and this bit is also"] # [doc = " not set, the current x-coordinate is a signed 16-bit delta"] # [doc = " vector."] const X_IS_SAME_OR_POSITIVE_X_SHORT_VECTOR = 0x10 ; # [doc = " Bit 5: This flag has two meanings, depending on how the"] # [doc = " Y_SHORT_VECTOR flag is set. If Y_SHORT_VECTOR is set, this bit"] # [doc = " describes the sign of the value, with 1 equalling positive and"] # [doc = " 0 negative. If Y_SHORT_VECTOR is not set and this bit is set,"] # [doc = " then the current y-coordinate is the same as the previous"] # [doc = " y-coordinate. If Y_SHORT_VECTOR is not set and this bit is also"] # [doc = " not set, the current y-coordinate is a signed 16-bit delta"] # [doc = " vector."] const Y_IS_SAME_OR_POSITIVE_Y_SHORT_VECTOR = 0x20 ; # [doc = " Bit 6: If set, contours in the glyph description may overlap."] # [doc = " Use of this flag is not required in OpenType — that is, it is"] # [doc = " valid to have contours overlap without having this flag set. It"] # [doc = " may affect behaviors in some platforms, however. (See the"] # [doc = " discussion of “Overlapping contours” in Apple’s"] # [doc = " specification for details regarding behavior in Apple"] # [doc = " platforms.) When used, it must be set on the first flag byte"] # [doc = " for the glyph. See additional details below."] const OVERLAP_SIMPLE = 0x40 ; } }

impl font_types::Scalar for SimpleGlyphFlags {
    type Raw = <u8 as font_types::Scalar>::Raw;
    fn to_raw(self) -> Self::Raw {
        self.bits().to_raw()
    }
    fn from_raw(raw: Self::Raw) -> Self {
        let t = <u8>::from_raw(raw);
        Self::from_bits_truncate(t)
    }
}

/// [CompositeGlyph](https://docs.microsoft.com/en-us/typography/opentype/spec/glyf#glyph-headers)
pub struct CompositeGlyph<'a> {
    header: zerocopy::LayoutVerified<&'a [u8], GlyphHeader>,
    flags: zerocopy::LayoutVerified<&'a [u8], BigEndian<CompositeGlyphFlags>>,
    glyph_index: zerocopy::LayoutVerified<&'a [u8], BigEndian<u16>>,
    offset_data: zerocopy::LayoutVerified<&'a [u8], [u8]>,
}

impl<'a> font_types::FontRead<'a> for CompositeGlyph<'a> {
    fn read(bytes: &'a [u8]) -> Option<Self> {
        let (header, bytes) =
            zerocopy::LayoutVerified::<_, GlyphHeader>::new_unaligned_from_prefix(bytes)?;
        let (flags , bytes) = zerocopy :: LayoutVerified :: < _ , BigEndian < CompositeGlyphFlags > > :: new_unaligned_from_prefix (bytes) ? ;
        let (glyph_index, bytes) =
            zerocopy::LayoutVerified::<_, BigEndian<u16>>::new_unaligned_from_prefix(bytes)?;
        let (offset_data, bytes) = (
            zerocopy::LayoutVerified::<_, [u8]>::new_slice_unaligned(bytes)?,
            0,
        );
        let _bytes = bytes;
        let result = CompositeGlyph {
            header,
            flags,
            glyph_index,
            offset_data,
        };
        Some(result)
    }
}

impl<'a> CompositeGlyph<'a> {
    pub fn header(&self) -> &GlyphHeader {
        &self.header
    }

    /// component flag
    pub fn flags(&self) -> CompositeGlyphFlags {
        self.flags.get()
    }

    /// glyph index of component
    pub fn glyph_index(&self) -> u16 {
        self.glyph_index.get()
    }

    pub fn offset_data(&self) -> &[u8] {
        &self.offset_data
    }
}

bitflags::bitflags! { # [doc = " Flags used in [CompositeGlyph]"] pub struct CompositeGlyphFlags : u16 { # [doc = " Bit 0: If this is set, the arguments are 16-bit (uint16 or"] # [doc = " int16); otherwise, they are bytes (uint8 or int8)."] const ARG_1_AND_2_ARE_WORDS = 0x0001 ; # [doc = " Bit 1: If this is set, the arguments are signed xy values;"] # [doc = " otherwise, they are unsigned point numbers."] const ARGS_ARE_XY_VALUES = 0x0002 ; # [doc = " Bit 2: If set and ARGS_ARE_XY_VALUES is also set, the xy values"] # [doc = " are rounded to the nearest grid line. Ignored if"] # [doc = " ARGS_ARE_XY_VALUES is not set."] const ROUND_XY_TO_GRID = 0x0004 ; # [doc = " Bit 3: This indicates that there is a simple scale for the"] # [doc = " component. Otherwise, scale = 1.0."] const WE_HAVE_A_SCALE = 0x0008 ; # [doc = " Bit 5: Indicates at least one more glyph after this one."] const MORE_COMPONENTS = 0x0020 ; # [doc = " Bit 6: The x direction will use a different scale from the y"] # [doc = " direction."] const WE_HAVE_AN_X_AND_Y_SCALE = 0x0040 ; # [doc = " Bit 7: There is a 2 by 2 transformation that will be used to"] # [doc = " scale the component."] const WE_HAVE_A_TWO_BY_TWO = 0x0080 ; # [doc = " Bit 8: Following the last component are instructions for the"] # [doc = " composite character."] const WE_HAVE_INSTRUCTIONS = 0x0100 ; # [doc = " Bit 9: If set, this forces the aw and lsb (and rsb) for the"] # [doc = " composite to be equal to those from this component glyph. This"] # [doc = " works for hinted and unhinted glyphs."] const USE_MY_METRICS = 0x0200 ; # [doc = " Bit 10: If set, the components of the compound glyph overlap."] # [doc = " Use of this flag is not required in OpenType — that is, it is"] # [doc = " valid to have components overlap without having this flag set."] # [doc = " It may affect behaviors in some platforms, however. (See"] # [doc = " Apple’s specification for details regarding behavior in Apple"] # [doc = " platforms.) When used, it must be set on the flag word for the"] # [doc = " first component. See additional remarks, above, for the similar"] # [doc = " OVERLAP_SIMPLE flag used in simple-glyph descriptions."] const OVERLAP_COMPOUND = 0x0400 ; # [doc = " Bit 11: The composite is designed to have the component offset"] # [doc = " scaled. Ignored if ARGS_ARE_XY_VALUES is not set."] const SCALED_COMPONENT_OFFSET = 0x0800 ; # [doc = " Bit 12: The composite is designed not to have the component"] # [doc = " offset scaled. Ignored if ARGS_ARE_XY_VALUES is not set."] const UNSCALED_COMPONENT_OFFSET = 0x1000 ; } }

impl font_types::Scalar for CompositeGlyphFlags {
    type Raw = <u16 as font_types::Scalar>::Raw;
    fn to_raw(self) -> Self::Raw {
        self.bits().to_raw()
    }
    fn from_raw(raw: Self::Raw) -> Self {
        let t = <u16>::from_raw(raw);
        Self::from_bits_truncate(t)
    }
}

pub enum Glyph<'a> {
    Simple(SimpleGlyph<'a>),
    Composite(CompositeGlyph<'a>),
}

impl<'a> font_types::FontRead<'a> for Glyph<'a> {
    fn read(bytes: &'a [u8]) -> Option<Self> {
        let version: BigEndian<i16> = font_types::FontRead::read(bytes)?;
        match version.get() {
            v if non_negative_i16(v) => Some(Self::Simple(font_types::FontRead::read(bytes)?)),
            v if i16::is_negative(v) => Some(Self::Composite(font_types::FontRead::read(bytes)?)),
            _other => {
                #[cfg(feature = "std")]
                {
                    eprintln!(
                        "unknown enum variant {:?} (table {})",
                        version,
                        stringify!(Glyph)
                    );
                }
                None
            }
        }
    }
}
