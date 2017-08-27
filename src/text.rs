use math::*;

use lodepng;
use gl;

static FONT_PNG: &[u8] = include_bytes!("../assets/BYond.png");

pub struct TextRenderer {
	tex_id: u32,
	pub width: u32,
	pub height: u32,
}

impl TextRenderer {
	pub fn new() -> Self {
		let tex_data = lodepng::decode32(&FONT_PNG).unwrap();

		let tex_id = unsafe {
			let mut id = 0u32;

			let (width, height) = (
				tex_data.width as i32,
				tex_data.height as i32
			);

			let test_data = [
				255u8, 0, 0, 255,
				255u8, 0, 0, 255,
				255u8, 255, 0, 255,
				255u8, 0, 0, 0,
			];

			gl::GenTextures(1, &mut id);
			gl::BindTexture(gl::TEXTURE_2D, id);
			gl::TexImage2D(gl::TEXTURE_2D, 0, gl::RGBA as _,
				width, height, 0, gl::RGBA as _, gl::UNSIGNED_BYTE,
				tex_data.buffer.as_ref().as_ptr() as *const _);

			gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as _);
			gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as _);
			gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as _);
			gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as _);

			id
		};

		TextRenderer {
			tex_id,
			width: tex_data.width as u32,
			height: tex_data.height as u32,
		}
	}

	pub unsafe fn draw(&self, text: &str, pos: Vec3, right: Vec3, center: bool) {
		self.draw_scale(text, pos, right, 1.0, center);
	}

	pub unsafe fn draw_scale(&self, text: &str, pos: Vec3, right: Vec3, scale: f32, center: bool) {
		let bs = text.as_bytes();

		let glyph_width = 12;
		let glyph_height = 14;
		let glyph_stride = 17;
		let margin = 1;
		let sz = 0.1 * scale;

		let real_tex_width = self.width as f32;
		let real_tex_height = self.height as f32;

		let tex_size = Vec2::new(real_tex_width, real_tex_height);
		let glyph_size = Vec2::splat(glyph_stride as f32) / tex_size;

		let str_width = glyph_stride as f32 * text.len() as f32 / tex_size.x as f32 * scale;
		let pos = if center { pos - right * str_width / 2.0 } else { pos };

		gl::Enable(gl::TEXTURE_2D);
		gl::Disable(gl::CULL_FACE);
		gl::Begin(gl::TRIANGLES);

		let up = Vec3::new(0.0, 1.0, 0.0);
		let zfix = right.cross(up) * 0.001;

		for (i, glyph) in bs.iter().enumerate() {
			let glyph = *glyph - 0x20;

			let x = (glyph as u32 % 16) * glyph_stride + margin;
			let y = (glyph as u32 / 16) * glyph_stride + margin;

			let bl = Vec2::new(x as f32, y as f32) / tex_size;

			let adv = right * i as f32 * glyph_stride as f32 / tex_size.x * scale;

			let verts = [
				pos + adv + zfix * i as f32,
				pos + adv + up * sz + zfix * i as f32,
				pos + adv + right *  sz + up * sz + zfix * i as f32,
				pos + adv + right *  sz + zfix * i as f32,
			];

			let uvs = [
				bl + Vec2::new(0.0, 1.0) * glyph_size,
				bl + Vec2::new(0.0, 0.0) * glyph_size,
				bl + Vec2::new(1.0, 0.0) * glyph_size,
				bl + Vec2::new(1.0, 1.0) * glyph_size,
			];

			gl::TexCoord2fv(&uvs[0].x);
			gl::Vertex3fv(&verts[0].x);
			gl::TexCoord2fv(&uvs[2].x);
			gl::Vertex3fv(&verts[2].x);
			gl::TexCoord2fv(&uvs[1].x);
			gl::Vertex3fv(&verts[1].x);

			gl::TexCoord2fv(&uvs[0].x);
			gl::Vertex3fv(&verts[0].x);
			gl::TexCoord2fv(&uvs[3].x);
			gl::Vertex3fv(&verts[3].x);
			gl::TexCoord2fv(&uvs[2].x);
			gl::Vertex3fv(&verts[2].x);
		}

		gl::End();

		gl::Disable(gl::TEXTURE_2D);
		gl::Enable(gl::CULL_FACE);	
	}
}