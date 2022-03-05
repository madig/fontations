//! Inspect a font, printing information about tables

use font_tables::{
    tables::{glyf::Bbox, TableProvider},
    FontRef,
};

fn main() {
    let path = std::env::args().nth(1).expect("missing path argument");
    let bytes = std::fs::read(path).unwrap();
    let font = FontRef::new(&bytes).unwrap();
    print_font_info(&font);
}

fn print_font_info(font: &FontRef) {
    let num_tables = font.table_directory.num_tables();
    println!("loaded {} tables", num_tables);
    for record in font.table_directory.table_records() {
        println!(
            "table {} at {:?} (len {})",
            record.tag.get(),
            record.offset.get(),
            record.len.get()
        );
    }

    let head = font.head().expect("missing head");
    let maxp = font.maxp().expect("missing maxp");
    if let Some(loca) = font.loca(maxp.num_glyphs(), head.index_to_loc_format() == 1) {
        let glyf = font.glyf().expect("missing glyf");
        let mut total_bbox = Bbox::default();
        //for gid in 0..maxp.num_glyphs() {
        for gid in 20..30u16 {
            let computed_bbox = glyf.compute_bbox(&loca, gid);
            let reported_bbox = loca
                .get(gid)
                .and_then(|off| glyf.resolve_glyph(off).map(|g| g.bbox()));
            if computed_bbox != reported_bbox {
                println!(
                    "glyph {}:\ncomputed: {:?}\nreported: {:?}",
                    gid, computed_bbox, reported_bbox
                );
            }
            total_bbox.union(computed_bbox.unwrap_or_default());
        }

        println!("total bbox:");
        println!(
            "computed: {}, {}, {}, {}",
            total_bbox.x_min, total_bbox.y_min, total_bbox.x_max, total_bbox.y_max
        );
        println!(
            "computed: {}, {}, {}, {}",
            head.x_min, head.y_min, head.x_max, head.y_max
        );
    }
}
