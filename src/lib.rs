use minifb::{Window, WindowOptions, MouseMode, MouseButton, Scale, Key};

use std::time::Instant;
use std::mem;

pub mod time;
pub mod gfx2d;

// TODO: ordering on this is format dependent?
#[derive(Debug, Clone)]
pub struct Pixel {
    pub b: u8,
    pub g: u8,
    pub r: u8,
    pub a: u8,
}

impl Pixel {
    pub fn rgb(r: u8, g: u8, b: u8) -> Self {
        Pixel{r:r, g:g, b:b, a:255}
    }

    pub fn rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Pixel{r:r, g:g, b:b, a:a}
    }

    // HACK?
    pub fn from_rgba_to_bgra(&mut self) {
        std::mem::swap(&mut self.b, &mut self.r);
    }
}

// Colors
pub const WHITE: Pixel                  = Pixel{r:255, g:255, b:255, a:255};
pub const GREY: Pixel                   = Pixel{r:192, g:192, b:192, a:255};
pub const DARK_GREY: Pixel              = Pixel{r:128, g:128, b:128, a:255};
pub const VERY_DARK_GREY: Pixel         = Pixel{r:64, g:64, b:64, a:255};
pub const RED: Pixel                    = Pixel{r:255, g:0, b:0, a:255};
pub const DARK_RED: Pixel               = Pixel{r:128, g:0, b:0, a:255};
pub const VERY_DARK_RED: Pixel          = Pixel{r:64, g:0, b:0, a:255};
pub const YELLOW: Pixel                 = Pixel{r:255, g:255, b:0, a:255};
pub const DARK_YELLOW: Pixel            = Pixel{r:128, g:128, b:0, a:255};
pub const VERY_DARK_YELLOW: Pixel       = Pixel{r:64, g:64, b:0, a:255};
pub const GREEN: Pixel                  = Pixel{r:0, g:255, b:0, a:255};
pub const DARK_GREEN: Pixel             = Pixel{r:0, g:128, b:0, a:255};
pub const VERY_DARK_GREEN: Pixel        = Pixel{r:0, g:64, b:0, a:255};
pub const CYAN: Pixel                   = Pixel{r:0, g:255, b:255, a:255};
pub const DARK_CYAN: Pixel              = Pixel{r:0, g:128, b:128, a:255};
pub const VERY_DARK_CYAN: Pixel         = Pixel{r:0, g:64, b:64, a:255};
pub const BLUE: Pixel                   = Pixel{r:0, g:0, b:255, a:255};
pub const DARK_BLUE: Pixel              = Pixel{r:0, g:0, b:128, a:255};
pub const VERY_DARK_BLUE: Pixel         = Pixel{r:0, g:0, b:64, a:255};
pub const MAGENTA: Pixel                = Pixel{r:255, g:0, b:255, a:255};
pub const DARK_MAGENTA: Pixel           = Pixel{r:128, g:0, b:128, a:255};
pub const VERY_DARK_MAGENTA: Pixel      = Pixel{r:64, g:0, b:64, a:255};
pub const BLACK: Pixel                  = Pixel{r:0, g:0, b:0, a:255};
pub const BLANK: Pixel                  = Pixel{r:0, g:0, b:0, a:0};

pub enum PixelMode {
    Normal, Mask, Alpha, Custom
}

#[derive(Debug, Clone, PartialEq)]
pub enum HWButton {
    Pressed,
    Released,
    Held,
}

#[derive(Debug, Clone)]
pub struct Sprite {
    pub width: usize,
    pub height: usize,
    pub data: Vec<Pixel>,
}

impl Sprite {
    pub fn new(width: usize, height: usize) -> Sprite {
        Sprite {
            width: width,
            height: height,
            data: vec![Pixel{r:0, g:0, b:0, a:0}; width * height],
        }
    }

    pub fn new_with_data(width: usize, height: usize, data: Vec<u8>) -> Sprite {
        unsafe {
            Sprite {
                width: width,
                height: height,
                data: std::mem::transmute::<Vec<u8>, Vec<Pixel>>(data),
            }
        }
    }

    pub fn from_rgba_to_bgra(&mut self) {
        for x in 0..self.width {
            for y in 0..self.height {
                self.data[y * self.width + x].from_rgba_to_bgra();
            }
        }
    }

    pub fn get_pixel(&self, x: i32, y: i32) -> Pixel {
        if x >= 0 && x < self.width as i32 && y >= 0 && y < self.height as i32 {
            self.data[(y * self.width as i32 + x) as usize].clone()
        } else {
            Pixel::rgba(0,0,0,0)
        }
    }

    pub fn set_pixel(&mut self, x: i32, y: i32, p: &Pixel) {
        if x >= 0 && x < self.width as i32 && y >= 0 && y < self.height as i32 {
            self.data[(y * self.width as i32 + x) as usize] = p.clone();
        }
    }

    pub fn sample(&self, x: f32, y: f32) -> Pixel {
        let sx = (x * self.width as f32) as i32;
        let sy = (y * self.height as f32) as i32;
        self.get_pixel(sx, sy)
    }

    pub fn get_data(&self) -> &[Pixel] {
        self.data.as_slice()
    }

    pub fn clear(&mut self, p: Pixel) {
        self.data = vec![p; self.width * self.height];
    }
}

#[allow(unused_variables)]
pub trait State {
    fn on_user_create(&mut self) -> bool { true }
    fn on_user_update(&mut self, pge: &mut PGE, elapsed_time: f32) -> bool { true }
    fn on_user_destroy(&mut self) {}
}

pub struct PGE {
    app_name: String,
    draw_target: Vec<Sprite>,
    current_draw_target: usize,
    window: Option<Window>,
    old_keys: Option<Vec<Key>>,
    keys: Option<Vec<Key>>,
    active: bool,
    screen_width: i32,
    screen_height: i32,
    pixel_width: i32,
    pixel_height: i32,
    pixel_x: f32,
    pixel_y: f32,
    mouse_pos_x: i32,
    mouse_pos_y: i32,
    font: Sprite,
    frame_timer: f32,
    frame_count: i32,
    mode: PixelMode,
    blend_factor: f32,
    func_pixel_mode: Option<fn(x: i32, y: i32, p1: &Pixel, p2: &Pixel)>
}

impl PGE {
    pub fn construct(name: &str, screen_w: usize, screen_h: usize, pixel_w: usize, pixel_h: usize) -> PGE {
        PGE {
            app_name: name.to_string(),
            draw_target: vec![Sprite::new(screen_w, screen_h)],
            current_draw_target: 0,
            window: None,
            old_keys: None,
            keys: None,
            active: true,
            screen_width: screen_w as i32 / pixel_w as i32,
            screen_height: screen_h as i32 / pixel_h as i32,
            pixel_width: pixel_w as i32,
            pixel_height: pixel_h as i32,
            pixel_x: 2.0 / screen_w as f32,
            pixel_y: 2.0 / screen_h as f32,
            mouse_pos_x: 0,
            mouse_pos_y: 0,
            font: PGE::construct_font_sheet(),
            frame_count: 0,
            frame_timer: 1.0,
            mode: PixelMode::Normal,
            blend_factor: 1.0,
            func_pixel_mode: None,
        }
    }

    pub fn start(&mut self, state: &mut State) {
        // Construct the window
        self.window = Some(Window::new(&self.app_name,
                                640,
                                480,
                                WindowOptions {
                                    scale: Scale::X2,
                                    borderless: false,
                                    resize: false,
                                    title: true
                                })
                                .unwrap_or_else(|e| {panic!("{}", e)}));

        if !state.on_user_create() {
            self.active = false;
        }


        let mut last_time = Instant::now();

        while self.active {
            let current_time = Instant::now();
            let elapsed = current_time - last_time;
            last_time = current_time;
            self.old_keys = self.keys.clone();

            let mut mpos = (0.0, 0.0);
            if let Some(window) = &mut self.window   {
                mpos = window.get_mouse_pos(MouseMode::Pass).unwrap();    
                self.keys = window.get_keys();
            }
            self.update_mouse(mpos.0 as i32, mpos.1 as i32);

            // Handle Frame Update
            let elapsed_time = time::duration_to_f64(elapsed) as f32;
            if !state.on_user_update(self, elapsed_time) {
                self.active = false;
            }

            // Display Graphics
            if let Some(window) = &mut self.window {
                unsafe {
                    window.update_with_buffer(
                        mem::transmute(self.draw_target[self.current_draw_target].data.as_slice())
                        ).unwrap_or_else(|e| {panic!("{}", e)});
                }
            }

            // Update title bar
            self.frame_timer += elapsed_time;
            self.frame_count += 1;
            if self.frame_timer >= 1.0 {
                self.frame_timer -= 1.0;
                if let Some(window) = &mut self.window {
                    let mut title = "OneLoneCoder.com - Pixel Game Engine - ".to_owned();
                    title += &self.app_name;
                    title += " - FPS: ";
                    title += &self.frame_count.to_string();

                    window.set_title(&title);
                    self.frame_count = 0;
                }
            }

            if let Some(window) = &mut self.window {
                if !window.is_open() {
                    self.active = false;
                }
            }
        }

        state.on_user_destroy();
    }

    // Hardware Interfaces

    pub fn is_focused(&mut self) -> bool {
        if let Some(window) = &mut self.window   {
            window.is_active()
        } else { false }
    }

    pub fn get_key(&mut self, k: Key) -> HWButton {
        if let Some(keys) = &self.keys {
            if let Some(old_keys) = &self.old_keys {
                if keys.contains(&k) && old_keys.contains(&k) {
                    return HWButton::Held
                } else if keys.contains(&k) && !old_keys.contains(&k) {
                    return HWButton::Pressed
                } else {
                    return HWButton::Released
                }
            }
        }
        HWButton::Released
    }

    pub fn get_mouse_x(&mut self) -> i32 {
        self.mouse_pos_x
    }

    pub fn get_mouse_y(&mut self) -> i32 {
        self.mouse_pos_y
    }

    pub fn get_mouse(&mut self, button: usize) -> HWButton {
        if let Some(window) = &self.window {
            match button {
                0 => if window.get_mouse_down(MouseButton::Left) {
                        return HWButton::Pressed
                    } else { return HWButton::Released },
                1 => if window.get_mouse_down(MouseButton::Middle) {
                        return HWButton::Pressed
                    } else { return HWButton::Released },
                2 => if window.get_mouse_down(MouseButton::Right) {
                        return HWButton::Pressed
                    } else { return HWButton::Released },
                _ => return HWButton::Released
            }
        } else {
            return HWButton::Released
        }
    }

    fn update_mouse(&mut self, x: i32, y: i32) {
        // Mouse coords come in screen space
		// But leave in pixel space
		self.mouse_pos_x = x / self.pixel_width;
		self.mouse_pos_y = y / self.pixel_height;

		if self.mouse_pos_x >= self.screen_width {
			self.mouse_pos_x = self.screen_width - 1;
        }
		if self.mouse_pos_y >= self.screen_height {
			self.mouse_pos_y = self.screen_height - 1;
        }

		if self.mouse_pos_x < 0
			{ self.mouse_pos_x = 0; }
		if self.mouse_pos_y < 0
			{ self.mouse_pos_y = 0; }
    }

    // Settings

    pub fn set_pixel_mode(&mut self, pm: PixelMode) {
        self.mode = pm;
    }

    // Draw Routines

    pub fn get_font(&mut self) -> Sprite {
        self.font.clone()
    }

    #[inline]
    pub fn draw(&mut self, x: i32, y: i32, p: &Pixel) {
        match self.mode {
            PixelMode::Normal => { 
                self.draw_target[self.current_draw_target].set_pixel(x, y, p); },
            PixelMode::Mask => {
                if p.a == 255 {
                    self.draw_target[self.current_draw_target].set_pixel(x, y, p);
                }
            },
            PixelMode::Alpha => {
                let d = self.draw_target[self.current_draw_target].get_pixel(x, y);
                let a = (p.a as f32 / 255.0) * self.blend_factor;
                let c = 1.0 - a;
                let r = a * p.r as f32 + c * d.r as f32;
                let g = a * p.g as f32 + c * d.g as f32;
                let b = a * p.b as f32 + c * d.b as f32;
                self.draw_target[self.current_draw_target].set_pixel(x, y, &Pixel::rgb(r as u8, g as u8, b as u8));
            },
            PixelMode::Custom => {
                if let Some(fpm) = self.func_pixel_mode {
                    fpm(x, y, &self.draw_target[self.current_draw_target].get_pixel(x, y), p);
                }
            }
        }
    }

    pub fn draw_line(&mut self, x1: i32, y1: i32, x2: i32, y2: i32, p: &Pixel) {
        let mut x = x1;
        let mut y = y1;
        let dx = i32::abs(x2 - x1);
        let dy = i32::abs(y2 - y1);
        let sx = if x1 < x2 { 1 } else { -1 };
        let sy = if y1 < y2 { 1 } else { -1 };
        let mut err = if dx > dy { dx / 2 } else { -dy / 2 };

        loop {
            //buffer[(y * (WIDTH as i32) + x) as usize] = color;
            self.draw(x, y, p);
            if x == x2 && y == y2 { break };
            if err > -dx {err = err - dy; x = x + sx; }
            if err < dy  {err = err + dx; y = y + sy; }        
        }
    }

    pub fn draw_circle(&mut self, x: i32, y: i32, radius: i32, p: &Pixel) {
        let mut x0 = 0;
		let mut y0 = radius;
		let mut d = 3 - 2 * radius;
		if radius <= 0 { return }

		while y0 >= x0 // only formulate 1/8 of circle
		{
			self.draw(x - x0, y - y0, p);//upper left left
			self.draw(x - y0, y - x0, p);//upper upper left
			self.draw(x + y0, y - x0, p);//upper upper right
			self.draw(x + x0, y - y0, p);//upper right right
			self.draw(x - x0, y + y0, p);//lower left left
			self.draw(x - y0, y + x0, p);//lower lower left
			self.draw(x + y0, y + x0, p);//lower lower right
			self.draw(x + x0, y + y0, p);//lower right right
			if d < 0 { d += 4 * x0 + 6; x0 += 1; }
			else { x0 += 1; y0 -= 1; d += 4 * (x0 - y0) + 10; }
		}
    }

    pub fn fill_circle(&mut self, x: i32, y: i32, radius: i32, p: &Pixel) {
        let mut x0 = 0;
		let mut y0 = radius;
		let mut d = 3 - 2 * radius;
		if radius <= 0 { return }

		while y0 >= x0
		{
			// Modified to draw scan-lines instead of edges
			self.draw_line(x - x0, y - y0, x + x0, y - y0, p);
			self.draw_line(x - y0, y - x0, x + y0, y - x0, p);
			self.draw_line(x - x0, y + y0, x + x0, y + y0, p);
			self.draw_line(x - y0, y + x0, x + y0, y + x0, p);
			if d < 0 { x0 += 1; d += 4 * x0 + 6;  }
			else { x0 += 1; y0 -= 1; d += 4 * (x0 - y0) + 10; }
		}
    }

    pub fn draw_rect(&mut self, x: i32, y: i32, w: i32, h: i32, p: &Pixel) {
        self.draw_line(x, y, x+w, y, p);
		self.draw_line(x+w, y, x+w, y+h, p);
		self.draw_line(x+w, y+h, x, y+h, p);
		self.draw_line(x, y+h, x, y, p);
    }

    pub fn fill_rect(&mut self, x: i32, y: i32, w: i32, h: i32, p: &Pixel) {
        let mut x1 = x;
        let mut y1 = y;
        let mut x2 = x + w;
		let mut y2 = y + h;

		if x < 0 { x1 = 0; }
		if x >= self.screen_width as i32 { x1 = self.screen_width as i32; }
		if y < 0 { y1 = 0; }
		if y >= self.screen_height as i32 { y1 = self.screen_height as i32; }

		if x2 < 0 { x2 = 0; }
		if x2 >= self.screen_width as i32 { x2 = self.screen_width as i32; }
		if y2 < 0 { y2 = 0; }
		if y2 >= self.screen_height as i32 { y2 = self.screen_height as i32; }

		for j in y1..y2 {
			for i in x1..x2 {
				self.draw(i, j, p);
            }
        }
    }

    pub fn draw_triangle(&mut self, x1: i32, y1: i32, x2: i32, y2: i32, x3: i32, y3: i32, p: &Pixel) {
        self.draw_line(x1, y1, x2, y2, p);
		self.draw_line(x2, y2, x3, y3, p);
		self.draw_line(x3, y3, x1, y1, p);
    }

    pub fn fill_triangle(&mut self, x1: i32, y1: i32, x2: i32, y2: i32, x3: i32, y3: i32, p: &Pixel) {
        // todo
    }

    pub fn draw_sprite(&mut self, x: i32, y: i32, sprite: &Sprite, scale: usize) {
        if scale > 1 {
            // todo
        } else {
            for j in 0..sprite.height as i32 {
                for i in 0..sprite.width as i32 {
                    self.draw(x + i, y + j, &sprite.get_pixel(i, j));
                }
            }
        }
    }

    pub fn draw_parital_sprite(&mut self, x: i32, y: i32, sprite: &Sprite, ox: i32, oy: i32, w: i32, h: i32, scale: usize) {
        if scale > 1 {
            // todo
        } else {
            for j in 0..h as i32 {
                for i in 0..w as i32 {
                    self.draw(x + i, y + j, &sprite.get_pixel(ox + i, oy + j));
                }
            }
        }
    }

    pub fn draw_string(&mut self, x: i32, y: i32, text: &str, col: &Pixel, scale: i32) {
        let mut sx: i32 = 0;
        let mut sy: i32 = 0;

        if col.a != 255 { self.set_pixel_mode(PixelMode::Alpha); }
        else { self.set_pixel_mode(PixelMode::Mask); }

        for c in text.chars() {
            if c == '\n' {
                sx = 0;
                sy += 8 * scale;
            } else {
                let mut ox: i32 = 15;
                let mut oy: i32 = 5;
                if c.is_ascii() {
                    ox = ((c as u32 - 32) % 16) as i32;
                    oy = ((c as u32 - 32) / 16) as i32;
                }
                if scale > 1 {
                    for j in 0..8 as i32 {
                        for i in 0..8 as i32 {
                            if self.font.get_pixel(i + ox * 8, j + oy * 8).r > 0 {
                                for js in 0..scale as i32 {
                                    for is in 0..scale as i32 {
                                        self.draw(x + sx + (i*scale) + is, y + sy + (j*scale) + js, &col);
                                    }
                                }
                            }
                        }
                    }
                } else {
                    for j in 0..8 as i32 {
                        for i in 0..8 as i32 {
                            if self.font.get_pixel(i + ox * 8, j + oy * 8).r > 0 {
                                self.draw(x + sx + i, y + sy + j, &col);
                            }
                        }
                    }
                }
                sx += 8 * scale;
            }
        }
    }

    pub fn clear(&mut self, p: &Pixel) {
        // NOTE: sloooooooow!!!!!
        /*
        for y in 0..self.screen_height {
            for x in 0..self.screen_width {
                self.draw_target[self.current_draw_target].set_pixel(x as i32, y as i32, &p);
            }
        }
        */
        // Much faster, but still might be slow?
        self.draw_target[self.current_draw_target].clear(p.clone());
    }

    fn construct_font_sheet() -> Sprite {
        let data = include_bytes!("font.png");
        let image = image::load_from_memory_with_format(data, image::ImageFormat::PNG).unwrap();
        let raw_image = image.raw_pixels();
        let mut pix_data: Vec<Pixel> = Vec::with_capacity(6144);
        let mut k = 0;
        for _ in 0..48 {
                for _ in 0..128 {
                    let r = raw_image[k];
                    let g = raw_image[k+1];
                    let b = raw_image[k+2];
                    let a = raw_image[k+3];
                    pix_data.push(Pixel::rgba(r, g, b, a));
                    k += 4;
                }
        }

        Sprite {
            width: 128,
            height: 48,
            data: pix_data
        }
    }
}