use image::{ImageBuffer, Rgba};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

/// Holiday and celebration theme categories for collages and stickers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum StickerCategory {
    All,
    Christmas,
    Halloween,
    FourthOfJuly,
    Thanksgiving,
    Easter,
    Valentine,
    NewYear,
    Birthday,
    MothersAndFathersDay,
    LunarNewYear,
    EverydayFun,
}

impl StickerCategory {
    pub fn all_filter_categories() -> &'static [StickerCategory] {
        &[
            StickerCategory::All,
            StickerCategory::Christmas,
            StickerCategory::Halloween,
            StickerCategory::FourthOfJuly,
            StickerCategory::Thanksgiving,
            StickerCategory::Easter,
            StickerCategory::Valentine,
            StickerCategory::NewYear,
            StickerCategory::Birthday,
            StickerCategory::MothersAndFathersDay,
            StickerCategory::LunarNewYear,
            StickerCategory::EverydayFun,
        ]
    }

    pub fn label(&self) -> &'static str {
        match self {
            StickerCategory::All => "All Stickers",
            StickerCategory::Christmas => "🎄 Christmas / Winter",
            StickerCategory::Halloween => "🎃 Halloween",
            StickerCategory::FourthOfJuly => "🎆 4th of July / USA",
            StickerCategory::Thanksgiving => "🦃 Thanksgiving / Autumn",
            StickerCategory::Easter => "🐰 Easter / Spring",
            StickerCategory::Valentine => "💖 Valentine's Day",
            StickerCategory::NewYear => "🥂 New Year's Eve",
            StickerCategory::Birthday => "🎂 Birthday & Celebration",
            StickerCategory::MothersAndFathersDay => "💐 Mother's & Father's Day",
            StickerCategory::LunarNewYear => "🧧 Lunar New Year",
            StickerCategory::EverydayFun => "⭐ Everyday Fun & Collage",
        }
    }

    pub fn short_label(&self) -> &'static str {
        match self {
            StickerCategory::All => "All",
            StickerCategory::Christmas => "Christmas",
            StickerCategory::Halloween => "Halloween",
            StickerCategory::FourthOfJuly => "4th of July",
            StickerCategory::Thanksgiving => "Thanksgiving",
            StickerCategory::Easter => "Easter",
            StickerCategory::Valentine => "Valentine",
            StickerCategory::NewYear => "New Year",
            StickerCategory::Birthday => "Birthday",
            StickerCategory::MothersAndFathersDay => "Mom & Dad",
            StickerCategory::LunarNewYear => "Lunar NY",
            StickerCategory::EverydayFun => "Everyday",
        }
    }
}

pub type StickerHolidayCategory = StickerCategory;

/// Metadata and asset definition for a holiday sticker.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StickerItem {
    pub id: String,
    pub name: String,
    pub category: StickerCategory,
    pub emoji: &'static str,
    pub primary_color: [u8; 4],
    pub accent_color: [u8; 4],
}

pub struct StickerCatalog;

static STICKER_ITEMS: std::sync::LazyLock<Vec<StickerItem>> = std::sync::LazyLock::new(|| {
    let raw: &[(&str, &str, StickerCategory, &'static str, [u8; 4], [u8; 4])] = &[
        // Halloween
        ("pumpkin", "Artistic Jack-o'-Lantern", StickerCategory::Halloween, "🎃", [255, 140, 0, 255], [75, 0, 130, 255]),
        ("ghost", "Friendly Autumn Ghost", StickerCategory::Halloween, "👻", [240, 248, 255, 255], [148, 0, 211, 255]),
        ("bat", "Watercolor Night Bat", StickerCategory::Halloween, "🦇", [47, 79, 79, 255], [255, 215, 0, 255]),
        ("candy", "Halloween Treat Sweets", StickerCategory::Halloween, "🍬", [255, 105, 180, 255], [255, 140, 0, 255]),
        ("spider_web", "Silver Spider Web", StickerCategory::Halloween, "🕸️", [200, 200, 200, 255], [40, 40, 50, 255]),
        ("skull", "Festive Folk Skull", StickerCategory::Halloween, "💀", [245, 245, 245, 255], [20, 20, 30, 255]),

        // Christmas
        ("xmas_tree", "Woodland Christmas Tree", StickerCategory::Christmas, "🎄", [34, 139, 34, 255], [255, 215, 0, 255]),
        ("santa", "Vintage Santa Claus", StickerCategory::Christmas, "🎅", [220, 20, 60, 255], [255, 255, 255, 255]),
        ("snowman", "Cozy Scarf Snowman", StickerCategory::Christmas, "⛄", [240, 248, 255, 255], [220, 20, 60, 255]),
        ("gift", "Festive Wrapped Gift", StickerCategory::Christmas, "🎁", [220, 20, 60, 255], [255, 215, 0, 255]),
        ("bell", "Golden Jingle Bells", StickerCategory::Christmas, "🔔", [255, 215, 0, 255], [220, 20, 60, 255]),
        ("snowflake", "Crystal Snowflake", StickerCategory::Christmas, "❄️", [173, 216, 230, 255], [255, 255, 255, 255]),
        ("candle", "Warm Holiday Candle", StickerCategory::Christmas, "🕯️", [255, 215, 0, 255], [200, 40, 40, 255]),

        // 4th of July / Independence Day
        ("eagle", "Majestic American Eagle", StickerCategory::FourthOfJuly, "🦅", [160, 82, 45, 255], [255, 215, 0, 255]),
        ("sparkler", "Festive Sparkler Wand", StickerCategory::FourthOfJuly, "🎆", [255, 215, 0, 255], [255, 255, 255, 255]),
        ("firecracker", "Patriotic Firecracker", StickerCategory::FourthOfJuly, "🧨", [255, 0, 0, 255], [255, 215, 0, 255]),
        ("star_glow", "Shining Liberty Star", StickerCategory::FourthOfJuly, "🌟", [255, 215, 0, 255], [30, 144, 255, 255]),
        ("popper", "Celebration Popper", StickerCategory::FourthOfJuly, "🎉", [255, 140, 0, 255], [50, 205, 50, 255]),
        ("star_gold", "Gold Star Rosette", StickerCategory::FourthOfJuly, "⭐", [255, 215, 0, 255], [220, 20, 60, 255]),

        // Thanksgiving
        ("turkey", "Harvest Autumn Turkey", StickerCategory::Thanksgiving, "🦃", [160, 82, 45, 255], [205, 133, 63, 255]),
        ("pie", "Rustic Pumpkin Pie", StickerCategory::Thanksgiving, "🥧", [244, 164, 96, 255], [210, 105, 30, 255]),
        ("autumn_leaf", "Golden Fall Foliage", StickerCategory::Thanksgiving, "🍂", [218, 112, 214, 255], [255, 69, 0, 255]),
        ("maple_leaf", "Watercolor Maple Leaf", StickerCategory::Thanksgiving, "🍁", [220, 60, 20, 255], [255, 140, 0, 255]),
        ("corn", "Harvest Indian Corn", StickerCategory::Thanksgiving, "🌽", [205, 133, 63, 255], [255, 215, 0, 255]),
        ("roast_leg", "Roast Feast Dish", StickerCategory::Thanksgiving, "🍗", [210, 105, 30, 255], [255, 255, 255, 255]),

        // Easter
        ("easter_bunny", "Fluffy Easter Bunny", StickerCategory::Easter, "🐰", [255, 250, 250, 255], [255, 192, 203, 255]),
        ("chick", "Hatching Spring Chick", StickerCategory::Easter, "🐣", [255, 255, 0, 255], [255, 140, 0, 255]),
        ("tulip", "Spring Tulip Bouquet", StickerCategory::Easter, "🌷", [255, 105, 180, 255], [34, 139, 34, 255]),
        ("butterfly", "Monarch Butterfly", StickerCategory::Easter, "🦋", [30, 144, 255, 255], [255, 20, 147, 255]),
        ("blossom", "Cherry Blossom Sprig", StickerCategory::Easter, "🌸", [255, 192, 203, 255], [255, 255, 255, 255]),
        ("egg", "Painted Easter Egg", StickerCategory::Easter, "🥚", [255, 220, 230, 255], [173, 216, 230, 255]),

        // Valentine's Day
        ("rose", "Lush Rose Bouquet", StickerCategory::Valentine, "🌹", [220, 20, 60, 255], [34, 139, 34, 255]),
        ("love_letter", "Vintage Love Letter", StickerCategory::Valentine, "💌", [255, 240, 245, 255], [220, 20, 60, 255]),
        ("red_heart", "Romantic Love Heart", StickerCategory::Valentine, "💖", [255, 20, 147, 255], [255, 105, 180, 255]),
        ("cupid_arrow", "Cupid Heart & Arrow", StickerCategory::Valentine, "💘", [255, 0, 128, 255], [255, 215, 0, 255]),
        ("sparkle_heart", "Shimmering Heart", StickerCategory::Valentine, "💖", [255, 20, 147, 255], [255, 255, 255, 255]),
        ("kiss", "Velvet Kiss Mark", StickerCategory::Valentine, "💋", [220, 20, 60, 255], [255, 105, 180, 255]),

        // New Year's
        ("champagne", "Celebration Cheers Toast", StickerCategory::NewYear, "🥂", [255, 215, 0, 255], [255, 255, 255, 255]),
        ("crown", "Golden Royal Crown", StickerCategory::NewYear, "👑", [255, 215, 0, 255], [220, 20, 60, 255]),
        ("confetti_ball", "Confetti Sparkle Ball", StickerCategory::NewYear, "🎊", [255, 105, 180, 255], [50, 205, 50, 255]),
        ("party_horn", "New Year Horn", StickerCategory::NewYear, "🎉", [255, 140, 0, 255], [50, 205, 50, 255]),

        // Birthday & Milestones
        ("bday_cake", "Celebration Birthday Cake", StickerCategory::Birthday, "🎂", [255, 182, 193, 255], [255, 215, 0, 255]),
        ("cupcake", "Sweet Berry Cupcake", StickerCategory::Birthday, "🧁", [255, 105, 180, 255], [245, 222, 179, 255]),
        ("bday_balloon", "Party Balloon Bouquet", StickerCategory::Birthday, "🎈", [220, 20, 60, 255], [255, 215, 0, 255]),
        ("lollipop", "Rainbow Swirl Lollipop", StickerCategory::Birthday, "🍭", [255, 69, 0, 255], [255, 255, 255, 255]),

        // Mother's & Father's Day
        ("bouquet", "Garden Flower Bouquet", StickerCategory::MothersAndFathersDay, "💐", [255, 105, 180, 255], [255, 215, 0, 255]),
        ("sunflower", "Golden Sunflower", StickerCategory::MothersAndFathersDay, "🌻", [255, 215, 0, 255], [139, 69, 19, 255]),
        ("trophy", "#1 Best Award Trophy", StickerCategory::MothersAndFathersDay, "🏆", [255, 215, 0, 255], [30, 144, 255, 255]),
        ("hibiscus", "Tropical Bloom", StickerCategory::MothersAndFathersDay, "🌺", [255, 20, 147, 255], [255, 215, 0, 255]),
        ("medal", "Golden Honor Medal", StickerCategory::MothersAndFathersDay, "🥇", [255, 215, 0, 255], [220, 20, 60, 255]),
        ("ribbon", "Festive Award Ribbon", StickerCategory::MothersAndFathersDay, "🎀", [255, 105, 180, 255], [255, 215, 0, 255]),

        // Lunar New Year
        ("dragon", "Auspicious Golden Dragon", StickerCategory::LunarNewYear, "🐉", [220, 20, 60, 255], [255, 215, 0, 255]),
        ("red_envelope", "Lucky Red Envelope", StickerCategory::LunarNewYear, "🧧", [255, 0, 0, 255], [255, 215, 0, 255]),
        ("red_lantern", "Silk Red Lantern", StickerCategory::LunarNewYear, "🏮", [220, 20, 60, 255], [255, 215, 0, 255]),
        ("gold_coin", "Fortune Gold Coin", StickerCategory::LunarNewYear, "🪙", [255, 215, 0, 255], [178, 34, 34, 255]),
        ("tangerine", "Prosperity Tangerine", StickerCategory::LunarNewYear, "🍊", [255, 140, 0, 255], [34, 139, 34, 255]),

        // Everyday Fun & Collage
        ("sun_face", "Radiant Sunshine", StickerCategory::EverydayFun, "🌞", [255, 215, 0, 255], [255, 140, 0, 255]),
        ("rainbow", "Dreamy Watercolor Rainbow", StickerCategory::EverydayFun, "🌈", [255, 105, 180, 255], [64, 224, 208, 255]),
        ("sparkles", "Magic Stardust Sparkles", StickerCategory::EverydayFun, "✨", [255, 215, 0, 255], [255, 255, 255, 255]),
        ("star_eyes", "Starstruck Smile", StickerCategory::EverydayFun, "🤩", [255, 215, 0, 255], [255, 69, 0, 255]),
        ("party_face", "Party Smile", StickerCategory::EverydayFun, "🥳", [255, 215, 0, 255], [138, 43, 226, 255]),
        ("heart_eyes", "Sweetheart Smile", StickerCategory::EverydayFun, "😍", [255, 215, 0, 255], [220, 20, 60, 255]),
    ];

    raw.iter()
        .map(|(id, name, cat, emoji, p_col, a_col)| StickerItem {
            id: id.to_string(),
            name: name.to_string(),
            category: *cat,
            emoji,
            primary_color: *p_col,
            accent_color: *a_col,
        })
        .collect()
});

impl StickerCatalog {
    pub fn all_stickers() -> &'static [StickerItem] {
        STICKER_ITEMS.as_slice()
    }

    /// Ensure all sticker image assets exist as transparent PNGs in `assets/stickers/`.
    pub fn ensure_sticker_assets_exist(assets_dir: &Path) {
        let stickers_dir = assets_dir.join("stickers");
        let _ = fs::create_dir_all(&stickers_dir);

        for item in Self::all_stickers() {
            let path = stickers_dir.join(format!("{}.png", item.id));
            if !path.exists() {
                let img = Self::generate_procedural_sticker_image(&item);
                let _ = img.save(&path);
            }
        }
    }

    /// Generates a fallback 256x256 anti-aliased RGBA sticker badge with badge borders.
    pub fn generate_procedural_sticker_image(item: &StickerItem) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
        let size = 256u32;
        let mut img = ImageBuffer::from_pixel(size, size, Rgba([0, 0, 0, 0]));

        let cx = size as f32 / 2.0;
        let cy = size as f32 / 2.0;
        let radius = size as f32 * 0.44;

        let [pr, pg, pb, _] = item.primary_color;
        let [ar, ag, ab, _] = item.accent_color;

        for y in 0..size {
            for x in 0..size {
                let dx = x as f32 - cx;
                let dy = y as f32 - cy;
                let dist = (dx * dx + dy * dy).sqrt();

                if dist <= radius {
                    let border_w = 12.0;
                    if dist >= radius - border_w {
                        let border_t = (dist - (radius - border_w)) / border_w;
                        let r = (ar as f32 * (1.0 - border_t) + 255.0 * border_t) as u8;
                        let g = (ag as f32 * (1.0 - border_t) + 255.0 * border_t) as u8;
                        let b = (ab as f32 * (1.0 - border_t) + 255.0 * border_t) as u8;
                        img.put_pixel(x, y, Rgba([r, g, b, 255]));
                    } else {
                        let t = dist / (radius - border_w);
                        let r = (pr as f32 * (1.0 - t * 0.35)) as u8;
                        let g = (pg as f32 * (1.0 - t * 0.35)) as u8;
                        let b = (pb as f32 * (1.0 - t * 0.35)) as u8;
                        img.put_pixel(x, y, Rgba([r, g, b, 245]));
                    }
                } else if dist <= radius + 2.5 {
                    let alpha = ((radius + 2.5 - dist) / 2.5 * 255.0).clamp(0.0, 255.0) as u8;
                    img.put_pixel(x, y, Rgba([ar, ag, ab, alpha]));
                }
            }
        }

        img
    }

    pub fn sticker_asset_path(assets_dir: &Path, sticker_id: &str) -> PathBuf {
        assets_dir.join("stickers").join(format!("{}.png", sticker_id))
    }
}
