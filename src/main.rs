use palette::rgb::Rgb;
use palette::Hsv;
use rand::prelude::*;
use std::fs;

enum PostFunction {
    LessFashionOverall,
    Fashion,
    LessFashion,
}

const DOWN: usize = 0;
const UP: usize = 1;
const RIGHT: usize = 2;
const LEFT: usize = 3;

#[derive(Copy, Clone)]
struct Address {
    row: usize,
    column: usize,
}

impl Address {
    fn new(row: usize, column: usize) -> Address {
        Address { row, column }
    }
}

struct Cell {
    color: u64,
    address: Address,
    neighbors: [Address; 4],
}

impl Cell {
    fn new(color: u64, row: usize, column: usize, width: usize, height: usize) -> Cell {
        let address = Address::new(row, column);
        let down = Address::new((row + 1) % height, column);
        let up = Address::new((row + height - 1) % height, column);
        let right = Address::new(row, (column + 1) % width);
        let left = Address::new(row, (column + width - 1) % width);
        let neighbors = [down, up, right, left];
        Cell {
            color,
            address,
            neighbors,
        }
    }
}

struct Alteration {
    color: u64,
    addresses: [Address; 4],
}

impl Alteration {
    fn new(color: u64, addresses: [Address; 4]) -> Alteration {
        Alteration { color, addresses }
    }
}

struct AutonImage {
    height: usize,
    width: usize,
    cells: Vec<Cell>,
}

impl AutonImage {
    fn new(height: usize, width: usize) -> AutonImage {
        let mut cells = Vec::with_capacity(height * width);
        for row in 0..height {
            for column in 0..width {
                cells.push(Cell::new(0, row, column, width, height));
            }
        }
        AutonImage {
            height,
            width,
            cells,
        }
    }
    fn fill_from(&mut self, source: &AutonImage) {
        for row in 0..source.height {
            for column in 0..source.width {
                self.get_mut_cell(row, column).color = source.get_cell(row, column).color;
            }
        }
    }
    fn get_cell(&self, row: usize, column: usize) -> &Cell {
        &self.cells[row * self.width + column]
    }
    fn get_mut_cell(&mut self, row: usize, column: usize) -> &mut Cell {
        &mut self.cells[row * self.width + column]
    }
    fn find_cell(&self, address: Address) -> &Cell {
        self.get_cell(address.row, address.column)
    }
    fn find_mut_cell(&mut self, address: Address) -> &mut Cell {
        self.get_mut_cell(address.row, address.column)
    }
    fn find_neighbor(&self, cell: &Cell, direction: usize) -> &Cell {
        self.find_cell(cell.neighbors[direction])
    }
    fn find_neighborhood<'a>(&'a self, cell: &'a Cell, height: usize, width: usize) -> Vec<&Cell> {
        let mut wildcard = cell;
        for _i in 0..width {
            wildcard = self.find_neighbor(wildcard, LEFT);
        }
        for _i in 0..height {
            wildcard = self.find_neighbor(wildcard, UP);
        }
        let neighborhood_height = height * 2 + 1;
        let neighborhood_width = width * 2 + 1;
        let neighborhood_cap = neighborhood_height * neighborhood_width;
        let mut neighborhood = Vec::with_capacity(neighborhood_cap);
        let mut direction = RIGHT;
        for row in 0..neighborhood_height {
            for column in 0..neighborhood_width {
                neighborhood.push(wildcard);
                /*
                    if (column === width * 2) {
                        wildcard = Visual.findNeighbor(space, wildcard, Visual.directions.Down);
                    } else {
                        wildcard = Visual.findNeighbor(space, wildcard, direction);
                    }
                */
                if column == width * 2 {
                    wildcard = self.find_neighbor(wildcard, DOWN);
                } else {
                    wildcard = self.find_neighbor(wildcard, DOWN);
                }
            }
            if direction == RIGHT {
                direction = LEFT;
            } else {
                direction = RIGHT;
            }
        }
        neighborhood
    }
    fn symmetry_address(
        &self,
        mut address: Address,
        vertically: bool,
        horizontally: bool,
    ) -> Address {
        if vertically {
            address.row = self.height - 1 - address.row;
        }
        if horizontally {
            address.column = self.width - 1 - address.column;
        }
        address
    }
    fn make_unique(&mut self, rng: &mut StdRng, color_modulus: u64) {
        let symmetry_height = self.height / 2;
        let symmetry_width = self.width / 2;
        let mut addresses: Vec<Address> = Vec::with_capacity(symmetry_height * symmetry_width);
        for row in 0..symmetry_height {
            for column in 0..symmetry_width {
                addresses.push(Address::new(row, column));
            }
        }
        let alterations_count = symmetry_height;
        let mut alterations: Vec<Alteration> = Vec::with_capacity(alterations_count);
        for _i in 0..alterations_count {
            let address_index = rng.gen_range(0..addresses.len());
            let address = addresses[address_index];
            let sym_1 = self.symmetry_address(address, false, true);
            let sym_2 = self.symmetry_address(address, true, false);
            let sym_3 = self.symmetry_address(address, true, true);
            let alt_addresses = [address, sym_1, sym_2, sym_3];
            let color = rng.gen_range(0..color_modulus);
            let alteration = Alteration::new(color, alt_addresses);
            alterations.push(alteration);
        }
        for alteration in alterations {
            for address in &alteration.addresses {
                let mut cell = self.find_mut_cell(*address);
                cell.color = alteration.color;
            }
        }
    }
}

struct Color {
    rgb: [u8; 3],
    hsv: [f32; 3],
}

impl Color {
    fn new(h: f32, s: f32, v: f32) -> Color {
        let hsv: [f32; 3] = [h, s, v];
        let mut rgb: [u8; 3] = [0, 0, 0];
        let palette_hsv = Hsv::new(h, s, v);
        let palette_rgb: Rgb = Rgb::from(palette_hsv);
        let rgb_tuple = palette_rgb.into_components();
        rgb[0] = (rgb_tuple.0 * 255.0) as u8;
        rgb[1] = (rgb_tuple.1 * 255.0) as u8;
        rgb[2] = (rgb_tuple.2 * 255.0) as u8;
        Color { hsv, rgb }
    }
}

struct Auton {
    height: usize,
    width: usize,
    front_a: AutonImage,
    back_a: AutonImage,
    front_b: AutonImage,
    back_b: AutonImage,
    colors: Vec<Color>,
    colors_count: usize,
    color_modulus: u64,
    rng: StdRng,
}

fn build_colors(rng: &mut StdRng, colors_count: usize) -> Vec<Color> {
    let mut colors = Vec::with_capacity(colors_count);
    let adder: f32 = 360f32 / colors_count as f32;
    let mut hue = rng.gen_range(0..360) as f32;
    for _i in 0..colors_count {
        hue += adder;
        hue %= 360.0;
        colors.push(Color::new(hue, 0.75, 1.0))
    }
    colors
}

fn at_least_one_not_zero(numbers: &Vec<u64>) -> bool {
    for number in numbers {
        if *number != 0 {
            return true;
        }
    }
    false
}

impl Auton {
    fn new(height: usize) -> Auton {
        let mut rng = StdRng::seed_from_u64(0);
        let width = height * 2;
        let mut front_a = AutonImage::new(height, width);
        let back_a = AutonImage::new(height, width);
        let front_b = AutonImage::new(height, width);
        let back_b = AutonImage::new(height, width);
        let colors_count = 16;
        let colors = build_colors(&mut rng, colors_count);
        let color_modulus = colors_count as u64;
        front_a.make_unique(&mut rng, color_modulus);
        Auton {
            height,
            width,
            front_a,
            back_a,
            front_b,
            back_b,
            colors_count,
            colors,
            color_modulus,
            rng,
        }
    }
    fn render(&mut self, directory: &str) {
        match fs::remove_dir_all(directory) {
            Ok(_) => (),
            Err(_) => (),
        }
        fs::create_dir_all(directory).unwrap();
        let pictures = 64;
        for picture in 0..pictures {
            println!("picture {}", picture);
            self.render_picture(directory, picture);
            self.iterate();
        }
        println!("picture {}", pictures);
        self.render_picture(directory, pictures);
    }
    fn iterate(&mut self) {
        self.back_a.fill_from(&self.front_a);
        for row in 0..self.height {
            for column in 0..self.width {
                let back_cell = self.back_a.get_cell(row, column);
                let a = back_cell.color;
                let b = self.back_a.find_neighbor(back_cell, DOWN).color;
                let c = self.back_a.find_neighbor(back_cell, UP).color;
                let d = self.back_a.find_neighbor(back_cell, RIGHT).color;
                let e = self.back_a.find_neighbor(back_cell, LEFT).color;
                let mut cell = self.front_a.get_mut_cell(row, column);
                cell.color = (a + b + c + d + e) % self.color_modulus;
                let numbers = vec![a, b, c, d, e];
                if at_least_one_not_zero(&numbers) {
                    if cell.color == 0 {
                        let mut front_b_cell = self.front_b.get_mut_cell(row, column);
                        front_b_cell.color += 1;
                        front_b_cell.color %= self.color_modulus;
                    }
                }
            }
        }
    }
    fn iterate_less_fashion_overall(&mut self) {
        let colors_counter: Vec<(u64, i64)> = Vec::with_capacity(self.color_modulus as usize);
    }
    fn render_picture(&self, directory: &str, picture: usize) {
        let mut image_buffer = image::ImageBuffer::new(self.width as u32, self.height as u32);
        // Iterate over the coordinates and pixels of the image
        for (x, y, pixel) in image_buffer.enumerate_pixels_mut() {
            let color_index = self.front_b.cells[y as usize * self.width + x as usize].color;
            let color = &self.colors[color_index as usize];
            *pixel = image::Rgb(color.rgb);
        }
        let path = format!("{}/{}_{}.png", directory, "picture", picture);
        image_buffer.save(path).unwrap();
    }
}

fn main() {
    println!("Hello, art!");
    let mut auton = Auton::new(64);
    auton.render("pictures");
}
