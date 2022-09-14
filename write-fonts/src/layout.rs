//! OpenType layout.

use std::collections::{BTreeMap, HashSet};

#[cfg(feature = "parsing")]
use read_fonts::FontRead;

pub mod gdef;
pub mod gpos;
mod value_record;

#[cfg(test)]
#[path = "./tests/layout.rs"]
#[cfg(feature = "parsing")]
mod spec_tests;

include!("../generated/generated_layout.rs");

/// A lookup table that is generic over the lookup type.
#[derive(Debug, Clone)]
pub struct Lookup<T> {
    pub lookup_flag: u16,
    pub subtables: Vec<OffsetMarker<T>>,
    pub mark_filtering_set: u16,
}

impl<T: LookupType + FontWrite> FontWrite for Lookup<T> {
    fn write_into(&self, writer: &mut TableWriter) {
        T::TYPE.write_into(writer);
        self.lookup_flag.write_into(writer);
        u16::try_from(self.subtables.len())
            .unwrap()
            .write_into(writer);
        self.subtables.write_into(writer);
        self.mark_filtering_set.write_into(writer);
    }
}

impl<T: Validate> Validate for Lookup<T> {
    fn validate_impl(&self, ctx: &mut ValidationCtx) {
        ctx.in_table("Lookup", |ctx| {
            ctx.in_field("subtables", |ctx| self.subtables.validate_impl(ctx))
        })
    }
}

/// An extension table that is generic over the subtable type.
#[derive(Debug, Clone)]
pub struct ExtensionSubtable<T> {
    pub extension_offset: OffsetMarker<T, 4>,
}

impl<T: LookupType + FontWrite> FontWrite for ExtensionSubtable<T> {
    fn write_into(&self, writer: &mut TableWriter) {
        1u16.write_into(writer);
        T::TYPE.write_into(writer);
        self.extension_offset.write_into(writer);
    }
}

impl<T: Validate> Validate for ExtensionSubtable<T> {
    fn validate_impl(&self, ctx: &mut ValidationCtx) {
        ctx.in_field("extension_offset", |ctx| {
            self.extension_offset.validate_impl(ctx)
        })
    }
}

#[cfg(feature = "parsing")]
impl<'a, U, T> FromObjRef<read_fonts::layout::TypedLookup<'a, U>> for Lookup<T>
where
    U: FontRead<'a>,
    T: FromTableRef<U> + 'static,
{
    fn from_obj_ref(from: &read_fonts::layout::TypedLookup<'a, U>, _data: FontData) -> Self {
        Lookup {
            lookup_flag: from.lookup_flag(),
            mark_filtering_set: from.mark_filtering_set(),
            subtables: from
                .subtable_offsets()
                .iter()
                .map(|off| {
                    let table_ref = from.get_subtable(off.get());
                    table_ref.into()
                })
                .collect(),
        }
    }
}

#[cfg(feature = "parsing")]
impl<'a, U, T> FromObjRef<read_fonts::layout::gpos::TypedExtension<'a, U>> for ExtensionSubtable<T>
where
    U: FontRead<'a>,
    T: FromTableRef<U> + 'static,
{
    fn from_obj_ref(
        from: &read_fonts::layout::gpos::TypedExtension<'a, U>,
        _data: FontData,
    ) -> Self {
        ExtensionSubtable {
            extension_offset: from.get().into(),
        }
    }
}

/// A utility trait for writing lookup tables
pub trait LookupType {
    /// The format type of this subtable.
    const TYPE: u16;
}

macro_rules! subtable_type {
    ($ty:ty, $val:expr) => {
        impl LookupType for $ty {
            const TYPE: u16 = $val;
        }
    };
}

subtable_type!(gpos::SinglePos, 1);
subtable_type!(gpos::PairPos, 2);
subtable_type!(gpos::CursivePosFormat1, 3);
subtable_type!(gpos::MarkBasePosFormat1, 4);
subtable_type!(gpos::MarkLigPosFormat1, 5);
subtable_type!(gpos::MarkMarkPosFormat1, 6);
subtable_type!(SequenceContext, 7);
subtable_type!(ChainedSequenceContext, 8);
subtable_type!(gpos::Extension, 9);

#[derive(Debug, Clone)]
pub enum FeatureParams {
    StylisticSet(StylisticSetParams),
    Size(SizeParams),
    CharacterVariant(CharacterVariantParams),
}

impl FontWrite for FeatureParams {
    fn write_into(&self, writer: &mut TableWriter) {
        match self {
            FeatureParams::StylisticSet(table) => table.write_into(writer),
            FeatureParams::Size(table) => table.write_into(writer),
            FeatureParams::CharacterVariant(table) => table.write_into(writer),
        }
    }
}

impl Validate for FeatureParams {
    fn validate_impl(&self, ctx: &mut ValidationCtx) {
        match self {
            Self::StylisticSet(table) => table.validate_impl(ctx),
            Self::Size(table) => table.validate_impl(ctx),
            Self::CharacterVariant(table) => table.validate_impl(ctx),
        }
    }
}

#[cfg(feature = "parsing")]
impl FromObjRef<read_fonts::layout::FeatureParams<'_>> for FeatureParams {
    fn from_obj_ref(from: &read_fonts::layout::FeatureParams, data: FontData) -> Self {
        use read_fonts::layout::FeatureParams as FromType;
        match from {
            FromType::Size(thing) => Self::Size(SizeParams::from_obj_ref(thing, data)),
            FromType::StylisticSet(thing) => {
                Self::StylisticSet(FromObjRef::from_obj_ref(thing, data))
            }
            FromType::CharacterVariant(thing) => {
                Self::CharacterVariant(FromObjRef::from_obj_ref(thing, data))
            }
        }
    }
}

#[cfg(feature = "parsing")]
impl FromTableRef<read_fonts::layout::FeatureParams<'_>> for FeatureParams {}

impl ClassDefFormat1 {
    fn iter(&self) -> impl Iterator<Item = (GlyphId, u16)> + '_ {
        self.class_value_array.iter().enumerate().map(|(i, cls)| {
            (
                GlyphId::new(self.start_glyph_id.to_u16().saturating_add(i as u16)),
                *cls,
            )
        })
    }
}

impl ClassRangeRecord {
    fn validate_glyph_range(&self, ctx: &mut ValidationCtx) {
        if self.start_glyph_id > self.end_glyph_id {
            ctx.report(format!(
                "start_glyph_id {} larger than end_glyph_id {}",
                self.start_glyph_id, self.end_glyph_id
            ));
        }
    }
}

impl ClassDefFormat2 {
    fn iter(&self) -> impl Iterator<Item = (GlyphId, u16)> + '_ {
        self.class_range_records.iter().flat_map(|rcd| {
            (rcd.start_glyph_id.to_u16()..=rcd.end_glyph_id.to_u16())
                .map(|gid| (GlyphId::new(gid), rcd.class))
        })
    }
}

impl ClassDef {
    pub fn iter(&self) -> impl Iterator<Item = (GlyphId, u16)> + '_ {
        let (one, two) = match self {
            Self::Format1(table) => (Some(table.iter()), None),
            Self::Format2(table) => (None, Some(table.iter())),
        };

        one.into_iter().flatten().chain(two.into_iter().flatten())
    }

    pub fn class_count(&self) -> u16 {
        //TODO: implement a good integer set!!
        self.iter()
            .map(|(_gid, cls)| cls)
            .chain(std::iter::once(0))
            .collect::<HashSet<_>>()
            .len()
            .try_into()
            .unwrap()
    }
}

impl CoverageFormat1 {
    fn iter(&self) -> impl Iterator<Item = GlyphId> + '_ {
        self.glyph_array.iter().copied()
    }

    fn len(&self) -> usize {
        self.glyph_array.len()
    }
}

impl CoverageFormat2 {
    fn iter(&self) -> impl Iterator<Item = GlyphId> + '_ {
        self.range_records
            .iter()
            .flat_map(|rcd| iter_gids(rcd.start_glyph_id, rcd.end_glyph_id))
    }

    fn len(&self) -> usize {
        self.range_records
            .iter()
            .map(|rcd| {
                rcd.end_glyph_id
                    .to_u16()
                    .saturating_sub(rcd.start_glyph_id.to_u16()) as usize
                    + 1
            })
            .sum()
    }
}

impl CoverageTable {
    pub fn iter(&self) -> impl Iterator<Item = GlyphId> + '_ {
        let (one, two) = match self {
            Self::Format1(table) => (Some(table.iter()), None),
            Self::Format2(table) => (None, Some(table.iter())),
        };

        one.into_iter().flatten().chain(two.into_iter().flatten())
    }

    pub fn len(&self) -> usize {
        match self {
            Self::Format1(table) => table.len(),
            Self::Format2(table) => table.len(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct ClassDefBuilder {
    pub items: BTreeMap<GlyphId, u16>,
}

#[derive(Debug, Default, PartialEq, Eq)]
pub struct CoverageTableBuilder {
    // invariant: is always sorted
    glyphs: Vec<GlyphId>,
}

impl FromIterator<GlyphId> for CoverageTableBuilder {
    fn from_iter<T: IntoIterator<Item = GlyphId>>(iter: T) -> Self {
        let glyphs = iter.into_iter().collect::<Vec<_>>();
        CoverageTableBuilder::from_glyphs(glyphs)
    }
}

impl CoverageTableBuilder {
    pub fn from_glyphs(mut glyphs: Vec<GlyphId>) -> Self {
        glyphs.sort_unstable();
        CoverageTableBuilder { glyphs }
    }

    /// Add a `GlyphId` to this coverage table.
    ///
    /// Returns the coverage index of the added glyph.
    ///
    /// If the glyph already exists, this returns its current index.
    pub fn add(&mut self, glyph: GlyphId) -> u16 {
        match self.glyphs.binary_search(&glyph) {
            Ok(ix) => ix as u16,
            Err(ix) => {
                self.glyphs.insert(ix, glyph);
                // if we're over u16::MAX glyphs, crash
                ix.try_into().unwrap()
            }
        }
    }

    pub fn build(self) -> CoverageTable {
        if should_choose_coverage_format_2(&self.glyphs) {
            CoverageTable::Format2(CoverageFormat2 {
                range_records: RangeRecord::iter_for_glyphs(&self.glyphs).collect(),
            })
        } else {
            CoverageTable::Format1(CoverageFormat1 {
                glyph_array: self.glyphs,
            })
        }
    }
}

impl FromIterator<(GlyphId, u16)> for ClassDefBuilder {
    fn from_iter<T: IntoIterator<Item = (GlyphId, u16)>>(iter: T) -> Self {
        Self {
            items: iter.into_iter().collect(),
        }
    }
}

impl ClassDefBuilder {
    fn prefer_format_1(&self) -> bool {
        // calculate our format2 size:
        let first = self.items.keys().next().map(|g| g.to_u16());
        let last = self.items.keys().next_back().map(|g| g.to_u16());
        let len_format1 = 3 + (last.unwrap_or_default() - first.unwrap_or_default()) as usize;
        let len_format2 = 4 + iter_class_ranges(&self.items).count() * 6;

        len_format1 < len_format2
    }

    pub fn build(&self) -> ClassDef {
        if self.prefer_format_1() {
            let first = self.items.keys().next().map(|g| g.to_u16()).unwrap_or(0);
            let last = self.items.keys().next_back().map(|g| g.to_u16());
            let class_value_array = (first..=last.unwrap_or_default())
                .map(|g| self.items.get(&GlyphId::new(g)).copied().unwrap_or(0))
                .collect();
            ClassDef::Format1(ClassDefFormat1 {
                start_glyph_id: self.items.keys().next().copied().unwrap_or(GlyphId::NOTDEF),
                class_value_array,
            })
        } else {
            ClassDef::Format2(ClassDefFormat2 {
                class_range_records: iter_class_ranges(&self.items).collect(),
            })
        }
    }
}

fn iter_class_ranges(
    values: &BTreeMap<GlyphId, u16>,
) -> impl Iterator<Item = ClassRangeRecord> + '_ {
    let mut iter = values.iter();
    let mut prev = None;

    #[allow(clippy::while_let_on_iterator)]
    std::iter::from_fn(move || {
        while let Some((gid, class)) = iter.next() {
            match prev.take() {
                None => prev = Some((*gid, *gid, *class)),
                Some((start, end, pclass)) if are_sequential(end, *gid) && pclass == *class => {
                    prev = Some((start, *gid, pclass))
                }
                Some((start_glyph_id, end_glyph_id, pclass)) => {
                    prev = Some((*gid, *gid, *class));
                    return Some(ClassRangeRecord {
                        start_glyph_id,
                        end_glyph_id,
                        class: pclass,
                    });
                }
            }
        }
        prev.take()
            .map(|(start_glyph_id, end_glyph_id, class)| ClassRangeRecord {
                start_glyph_id,
                end_glyph_id,
                class,
            })
    })
}

//TODO: this can be fancier; we probably want to do something like find the
// percentage of glyphs that are in contiguous ranges, or something?
fn should_choose_coverage_format_2(glyphs: &[GlyphId]) -> bool {
    let format2_len = 4 + RangeRecord::iter_for_glyphs(glyphs).count() * 6;
    let format1_len = 4 + glyphs.len() * 2;
    format2_len < format1_len
}

impl RangeRecord {
    /// An iterator over records for this array of glyphs.
    ///
    /// # Note
    ///
    /// this function expects that glyphs are already sorted.
    pub fn iter_for_glyphs(glyphs: &[GlyphId]) -> impl Iterator<Item = RangeRecord> + '_ {
        let mut cur_range = glyphs.first().copied().map(|g| (g, g));
        let mut len = 0u16;
        let mut iter = glyphs.iter().skip(1).copied();

        #[allow(clippy::while_let_on_iterator)]
        std::iter::from_fn(move || {
            while let Some(glyph) = iter.next() {
                match cur_range {
                    None => return None,
                    Some((a, b)) if are_sequential(b, glyph) => cur_range = Some((a, glyph)),
                    Some((a, b)) => {
                        let result = RangeRecord {
                            start_glyph_id: a,
                            end_glyph_id: b,
                            start_coverage_index: len,
                        };
                        cur_range = Some((glyph, glyph));
                        len += 1 + b.to_u16().saturating_sub(a.to_u16());
                        return Some(result);
                    }
                }
            }
            cur_range
                .take()
                .map(|(start_glyph_id, end_glyph_id)| RangeRecord {
                    start_glyph_id,
                    end_glyph_id,
                    start_coverage_index: len,
                })
        })
    }
}

#[cfg(feature = "parsing")]
fn convert_delta_format(from: read_fonts::layout::DeltaFormat) -> DeltaFormat {
    match from as u16 {
        0x0002 => DeltaFormat::Local4BitDeltas,
        0x0003 => DeltaFormat::Local8BitDeltas,
        0x8000 => DeltaFormat::VariationIndex,
        _ => DeltaFormat::Local2BitDeltas,
    }
}

impl Default for DeltaFormat {
    fn default() -> Self {
        Self::Local2BitDeltas
    }
}

fn iter_gids(gid1: GlyphId, gid2: GlyphId) -> impl Iterator<Item = GlyphId> {
    (gid1.to_u16()..=gid2.to_u16()).map(GlyphId::new)
}

fn are_sequential(gid1: GlyphId, gid2: GlyphId) -> bool {
    gid2.to_u16().saturating_sub(gid1.to_u16()) == 1
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[should_panic(expected = "array excedes max length")]
    fn array_len_smoke_test() {
        let table = ScriptList {
            script_records: vec![ScriptRecord {
                script_tag: Tag::new(b"hihi"),
                script_offset: OffsetMarker::new(Script {
                    default_lang_sys_offset: NullableOffsetMarker::new(None),
                    lang_sys_records: vec![LangSysRecord {
                        lang_sys_tag: Tag::new(b"coco"),
                        lang_sys_offset: OffsetMarker::new(LangSys {
                            required_feature_index: 0xffff,
                            feature_indices: vec![69; (u16::MAX) as usize + 5],
                        }),
                    }],
                }),
            }],
        };

        table.validate().unwrap();
    }

    #[test]
    #[should_panic(expected = "larger than end_glyph_id")]
    fn validate_classdef_ranges() {
        let classdef = ClassDefFormat2 {
            class_range_records: vec![ClassRangeRecord {
                start_glyph_id: GlyphId::new(12),
                end_glyph_id: GlyphId::new(3),
                class: 7,
            }],
        };

        classdef.validate().unwrap();
    }

    #[test]
    fn classdef_format() {
        let builder: ClassDefBuilder = [(3u16, 4u16), (4, 6), (5, 1), (9, 5), (10, 2), (11, 3)]
            .map(|(gid, cls)| (GlyphId::new(gid), cls))
            .into_iter()
            .collect();

        assert!(builder.prefer_format_1());

        let builder: ClassDefBuilder = [(1u16, 1u16), (3, 4), (9, 5), (10, 2), (11, 3)]
            .map(|(gid, cls)| (GlyphId::new(gid), cls))
            .into_iter()
            .collect();

        assert!(builder.prefer_format_1());
    }
}
