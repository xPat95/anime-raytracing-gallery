use raylib::prelude::*;

pub const GALLERY_ENTRIES: [GalleryEntry; 5] = [
    GalleryEntry::new(
        "black_clover_skull",
        "assets/logos/black_clover_logo.png",
        "assets/previews/skull.png",
    ),
    GalleryEntry::new(
        "shenlong",
        "assets/logos/Dragon_Ball_logo.png",
        "assets/previews/shenlong.png",
    ),
    GalleryEntry::new(
        "kurama",
        "assets/logos/naruto_logo.png",
        "assets/previews/kurama.png",
    ),
    GalleryEntry::new(
        "pochita",
        "assets/logos/chainsawman_logo.png",
        "assets/previews/pochita.png",
    ),
    GalleryEntry::new(
        "lapras",
        "assets/logos/pokemon_logo.png",
        "assets/previews/lapras.png",
    ),
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AppState {
    Cover,
    Gallery,
    Scene,
}

#[derive(Clone, Copy)]
pub struct GalleryEntry {
    pub scene_id: &'static str,
    pub logo_path: &'static str,
    pub preview_path: &'static str,
}

impl GalleryEntry {
    const fn new(
        scene_id: &'static str,
        logo_path: &'static str,
        preview_path: &'static str,
    ) -> Self {
        Self {
            scene_id,
            logo_path,
            preview_path,
        }
    }
}

pub struct GalleryState {
    current_page: usize,
}

impl GalleryState {
    pub fn new() -> Self {
        Self { current_page: 0 }
    }

    pub fn current_page(&self) -> usize {
        self.current_page
    }

    pub fn current_entry(&self) -> GalleryEntry {
        GALLERY_ENTRIES[self.current_page]
    }

    pub fn previous(&mut self) {
        self.current_page = self.current_page.saturating_sub(1);
    }

    pub fn next(&mut self) {
        self.current_page = (self.current_page + 1).min(GALLERY_ENTRIES.len() - 1);
    }
}

pub struct GalleryAsset {
    logo: Texture2D,
    preview: Texture2D,
}

pub struct UiAssets {
    entries: Vec<GalleryAsset>,
}

impl UiAssets {
    pub fn load(raylib: &mut RaylibHandle, thread: &RaylibThread) -> Result<Self, String> {
        let mut entries = Vec::with_capacity(GALLERY_ENTRIES.len());
        for entry in GALLERY_ENTRIES {
            let logo = raylib
                .load_texture(thread, entry.logo_path)
                .map_err(|error| format!("could not load '{}': {error}", entry.logo_path))?;
            let preview = raylib
                .load_texture(thread, entry.preview_path)
                .map_err(|error| format!("could not load '{}': {error}", entry.preview_path))?;
            logo.set_texture_filter(thread, TextureFilter::TEXTURE_FILTER_BILINEAR);
            preview.set_texture_filter(thread, TextureFilter::TEXTURE_FILTER_BILINEAR);
            entries.push(GalleryAsset { logo, preview });
        }
        Ok(Self { entries })
    }

    fn current(&self, page: usize) -> &GalleryAsset {
        &self.entries[page]
    }
}

#[derive(Clone, Copy, Default)]
pub struct GalleryLayout {
    pub previous_button: Rectangle,
    pub next_button: Rectangle,
    pub preview_button: Rectangle,
}

pub fn draw_cover(drawing: &mut RaylibDrawHandle<'_>) -> Rectangle {
    let screen_width = drawing.get_screen_width() as f32;
    let screen_height = drawing.get_screen_height() as f32;
    let height = (screen_height * 0.72).clamp(360.0, 680.0);
    let width = (height * 0.68).min(screen_width * 0.72);
    let cover = Rectangle::new(
        (screen_width - width) * 0.5,
        (screen_height - height) * 0.5,
        width,
        height,
    );

    drawing.clear_background(Color::new(39, 29, 22, 255));
    drawing.draw_rectangle_rec(
        Rectangle::new(cover.x + 12.0, cover.y + 14.0, cover.width, cover.height),
        Color::new(20, 14, 10, 180),
    );
    drawing.draw_rectangle_rec(cover, Color::new(91, 49, 32, 255));
    draw_pixel_border(drawing, cover, Color::new(48, 26, 19, 255), 10.0);
    let inset = inset(cover, 28.0);
    draw_pixel_border(drawing, inset, Color::new(151, 94, 53, 255), 5.0);
    drawing.draw_rectangle(
        (cover.x + 20.0) as i32,
        cover.y as i32,
        18,
        cover.height as i32,
        Color::new(61, 31, 24, 255),
    );

    let title_size = if cover.height < 500.0 { 28 } else { 38 };
    let title_y = cover.y + cover.height * 0.27;
    for (index, line) in ["MINECRAFT", "RAYTRACING", "ART GALLERY"]
        .iter()
        .enumerate()
    {
        draw_centered_text(
            drawing,
            line,
            cover.x + cover.width * 0.5,
            title_y + index as f32 * (title_size as f32 + 16.0),
            title_size,
            Color::new(239, 215, 157, 255),
        );
    }
    draw_centered_text(
        drawing,
        "Click to open",
        cover.x + cover.width * 0.5,
        cover.y + cover.height - 72.0,
        20,
        Color::new(219, 187, 126, 255),
    );
    cover
}

pub fn draw_gallery(
    drawing: &mut RaylibDrawHandle<'_>,
    assets: &UiAssets,
    gallery: &GalleryState,
    display_name: &str,
) -> GalleryLayout {
    let screen_width = drawing.get_screen_width() as f32;
    let screen_height = drawing.get_screen_height() as f32;
    drawing.clear_background(Color::new(49, 35, 25, 255));

    let width = (screen_width * 0.88).min(1260.0);
    let height = (screen_height * 0.78).min(width * 0.58).max(390.0);
    let width = width.min(height * 1.75);
    let book = Rectangle::new(
        (screen_width - width) * 0.5,
        (screen_height - height) * 0.5,
        width,
        height,
    );
    let half = book.width * 0.5;
    let left = Rectangle::new(book.x, book.y, half, book.height);
    let right = Rectangle::new(book.x + half, book.y, half, book.height);
    drawing.draw_rectangle_rec(
        Rectangle::new(book.x + 10.0, book.y + 12.0, book.width, book.height),
        Color::new(20, 14, 10, 180),
    );
    drawing.draw_rectangle_rec(book, Color::new(91, 49, 32, 255));
    drawing.draw_rectangle_rec(inset(left, 11.0), Color::new(231, 213, 166, 255));
    drawing.draw_rectangle_rec(inset(right, 11.0), Color::new(238, 220, 174, 255));
    draw_pixel_border(drawing, book, Color::new(58, 31, 21, 255), 8.0);
    drawing.draw_rectangle(
        (book.x + half - 7.0) as i32,
        (book.y + 8.0) as i32,
        14,
        (book.height - 16.0) as i32,
        Color::new(105, 67, 43, 255),
    );

    let compact = book.height < 520.0;
    let title_size = if compact { 24 } else { 32 };
    draw_centered_text(
        drawing,
        &display_name.to_uppercase(),
        left.x + left.width * 0.5,
        left.y + 52.0,
        title_size,
        Color::new(50, 35, 24, 255),
    );

    let asset = assets.current(gallery.current_page());
    let logo_bounds = Rectangle::new(
        left.x + left.width * 0.14,
        left.y + left.height * 0.28,
        left.width * 0.72,
        left.height * 0.34,
    );
    draw_texture_contained(drawing, &asset.logo, logo_bounds, Color::WHITE);

    let preview_button = Rectangle::new(
        right.x + right.width * 0.10,
        right.y + right.height * 0.12,
        right.width * 0.80,
        right.height * 0.68,
    );
    drawing.draw_rectangle_rec(preview_button, Color::new(70, 47, 31, 255));
    draw_texture_contained(
        drawing,
        &asset.preview,
        inset(preview_button, 6.0),
        Color::WHITE,
    );
    draw_centered_text(
        drawing,
        "Click to view",
        right.x + right.width * 0.5,
        right.y + right.height - 70.0,
        20,
        Color::new(50, 35, 24, 255),
    );

    let button_size = if compact { 40.0 } else { 48.0 };
    let previous_button = Rectangle::new(
        left.x + 34.0,
        left.y + left.height - button_size - 28.0,
        button_size,
        button_size,
    );
    let next_button = Rectangle::new(
        right.x + right.width - button_size - 34.0,
        right.y + right.height - button_size - 28.0,
        button_size,
        button_size,
    );
    draw_arrow_button(drawing, previous_button, false, gallery.current_page() > 0);
    draw_arrow_button(
        drawing,
        next_button,
        true,
        gallery.current_page() + 1 < GALLERY_ENTRIES.len(),
    );
    draw_centered_text(
        drawing,
        &format!(
            "Page {} of {}",
            gallery.current_page() + 1,
            GALLERY_ENTRIES.len()
        ),
        left.x + left.width * 0.5,
        left.y + left.height - 58.0,
        18,
        Color::new(66, 45, 30, 255),
    );

    GalleryLayout {
        previous_button,
        next_button,
        preview_button,
    }
}

pub fn book_button(screen_width: i32, screen_height: i32) -> Rectangle {
    let size = 54.0_f32
        .min(screen_width as f32 * 0.12)
        .min(screen_height as f32 * 0.12);
    Rectangle::new(18.0, 18.0, size.max(42.0), size.max(42.0))
}

pub fn draw_book_button(drawing: &mut RaylibDrawHandle<'_>, bounds: Rectangle) {
    drawing.draw_rectangle_rec(bounds, Color::new(76, 42, 28, 235));
    draw_pixel_border(drawing, bounds, Color::new(231, 213, 166, 255), 3.0);
    let center_x = bounds.x + bounds.width * 0.5;
    let top = bounds.y + bounds.height * 0.27;
    let bottom = bounds.y + bounds.height * 0.73;
    drawing.draw_line_ex(
        Vector2::new(center_x, top),
        Vector2::new(center_x, bottom),
        2.0,
        Color::new(70, 45, 28, 255),
    );
    drawing.draw_rectangle_rec(
        Rectangle::new(
            bounds.x + 10.0,
            top,
            bounds.width * 0.5 - 10.0,
            bottom - top,
        ),
        Color::new(238, 220, 174, 255),
    );
    drawing.draw_rectangle_rec(
        Rectangle::new(center_x, top, bounds.width * 0.5 - 10.0, bottom - top),
        Color::new(231, 213, 166, 255),
    );
    drawing.draw_line_ex(
        Vector2::new(center_x, top),
        Vector2::new(center_x, bottom),
        2.0,
        Color::new(91, 49, 32, 255),
    );
}

pub fn contains(rectangle: Rectangle, point: Vector2) -> bool {
    point.x >= rectangle.x
        && point.x <= rectangle.x + rectangle.width
        && point.y >= rectangle.y
        && point.y <= rectangle.y + rectangle.height
}

fn draw_arrow_button(
    drawing: &mut RaylibDrawHandle<'_>,
    bounds: Rectangle,
    points_right: bool,
    enabled: bool,
) {
    let color = if enabled {
        Color::new(76, 48, 29, 255)
    } else {
        Color::new(164, 146, 110, 255)
    };
    let center = Vector2::new(
        bounds.x + bounds.width * 0.5,
        bounds.y + bounds.height * 0.5,
    );
    let direction = if points_right { 1.0 } else { -1.0 };
    drawing.draw_triangle(
        Vector2::new(center.x + direction * 12.0, center.y),
        Vector2::new(center.x - direction * 8.0, center.y - 13.0),
        Vector2::new(center.x - direction * 8.0, center.y + 13.0),
        color,
    );
}

fn draw_texture_contained(
    drawing: &mut RaylibDrawHandle<'_>,
    texture: &Texture2D,
    bounds: Rectangle,
    tint: Color,
) {
    let scale = (bounds.width / texture.width as f32).min(bounds.height / texture.height as f32);
    let width = texture.width as f32 * scale;
    let height = texture.height as f32 * scale;
    let destination = Rectangle::new(
        bounds.x + (bounds.width - width) * 0.5,
        bounds.y + (bounds.height - height) * 0.5,
        width,
        height,
    );
    drawing.draw_texture_pro(
        texture,
        Rectangle::new(0.0, 0.0, texture.width as f32, texture.height as f32),
        destination,
        Vector2::zero(),
        0.0,
        tint,
    );
}

fn draw_centered_text(
    drawing: &mut RaylibDrawHandle<'_>,
    text: &str,
    center_x: f32,
    y: f32,
    size: i32,
    color: Color,
) {
    let width = drawing.measure_text(text, size);
    drawing.draw_text(text, center_x as i32 - width / 2, y as i32, size, color);
}

fn draw_pixel_border(
    drawing: &mut RaylibDrawHandle<'_>,
    rectangle: Rectangle,
    color: Color,
    thickness: f32,
) {
    drawing.draw_rectangle_lines_ex(rectangle, thickness, color);
}

fn inset(rectangle: Rectangle, amount: f32) -> Rectangle {
    Rectangle::new(
        rectangle.x + amount,
        rectangle.y + amount,
        rectangle.width - amount * 2.0,
        rectangle.height - amount * 2.0,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn entries_keep_the_required_order() {
        let ids: Vec<_> = GALLERY_ENTRIES.iter().map(|entry| entry.scene_id).collect();
        assert_eq!(
            ids,
            [
                "black_clover_skull",
                "shenlong",
                "kurama",
                "pochita",
                "lapras"
            ]
        );
    }

    #[test]
    fn navigation_starts_at_zero_and_stays_in_bounds() {
        let mut gallery = GalleryState::new();
        assert_eq!(gallery.current_page(), 0);
        gallery.previous();
        assert_eq!(gallery.current_page(), 0);
        for _ in 0..10 {
            gallery.next();
        }
        assert_eq!(gallery.current_page(), GALLERY_ENTRIES.len() - 1);
        for _ in 0..10 {
            gallery.previous();
        }
        assert_eq!(gallery.current_page(), 0);
    }
}
