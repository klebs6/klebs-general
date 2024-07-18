crate::ix!();

pub struct PdfApp {
    document:       Arc<PopplerDocument>,
    current_page:   usize,
    crop_rect:      Rect,
    bounding_boxes: Vec<Rect>,
}

impl PdfApp {

    pub fn new(document_path: &str) -> Self {
        let password = None;
        let document = PopplerDocument::new_from_file(document_path, password).expect("Failed to open PDF");
        Self {
            document: Arc::new(document),
            current_page: 0,
            crop_rect: Rect::EVERYTHING,
            bounding_boxes: Vec::new(),
        }
    }

    pub fn ui(&mut self, ctx: &Context) -> Result<(),PdfAppError> {

        egui::CentralPanel::default().show(ctx, |ui| {

            ui.label("PDF Viewer");

            // Render current page
            let page = self.document.pages().nth(self.current_page).expect("Failed to get page");

            let (width, height) = page.get_size();

            let scale_factor = 2.0; // Adjust the scale factor as needed

            // Create a Cairo image surface to render the page
            let surface = ImageSurface::create(Format::ARgb32, (width * scale_factor) as i32, (height * scale_factor) as i32)
                .expect("Failed to create surface");

            let cairo_ctx = CairoContext::new(&surface).expect("could not create CairoContext");

            // Render the page to the Cairo context
            page.render(&cairo_ctx);

            // Convert the Cairo surface to an image buffer
            let mut data = vec![0u8; surface.stride() as usize * surface.height() as usize];

            surface
                .write_to_png(&mut data)
                .expect("Failed to write to PNG");

            let image_buffer: ImageBuffer<Rgba<u8>, Vec<u8>> = ImageBuffer::from_raw(
                surface.width() as u32,
                surface.height() as u32,
                data,
            ).expect("Failed to create image");

            let dynamic_image = DynamicImage::ImageRgba8(image_buffer);

            let image_data = egui::ColorImage::from_rgba_unmultiplied(
                [surface.width() as usize, surface.height() as usize],
                &dynamic_image.to_rgba8(),
            );

            let texture_id = ui.ctx().load_texture("page", image_data, Default::default());

            // Use the correct ui.image method
            ui.image(&texture_id);

            // Crop and bounding box selection
            ui.horizontal(|ui| {
                if ui.button("Set Crop").clicked() {
                    // Set crop rect logic here
                }

                if ui.button("Add Bounding Box").clicked() {
                    // Add bounding box logic here
                }

                if ui.button("Next Page").clicked() {
                    if self.current_page < self.document.pages().count() - 1 {
                        self.current_page += 1;
                    }
                }

                if ui.button("Previous Page").clicked() {
                    if self.current_page > 0 {
                        self.current_page -= 1;
                    }
                }
            });
        });

        Ok(())
    }
}
