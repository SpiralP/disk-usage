fn main() {
    parceljs_builder::Builder {
        yarn: true,
        ..Default::default()
    }
    .build();
}
