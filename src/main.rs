use std::{collections::HashMap, path::Path, io::Write};

use asefile::AsepriteFile;
use noise::{NoiseFn, Perlin};
use macroquad::prelude::*;

const BLOCK_FULL_SIZE: f32 = 84.0;
const WORLD_SIZE_X: usize = 128;
const WORLD_SIZE_Y: usize = 64;
const WORLD_SIZE_Z: usize = WORLD_SIZE_X;

const MAGIC_NUMBER: f64 = 0.025;
const ANOTHER_MAGIC_NUMBER: f64 = 6.0;

// UPDATE THIS NUMBER WITH THE NUMBER OF HOURS SPENT OPTIMIZING THIS PIECE OF SHIT
const _HOURS_SPENT: u32 = 4;

#[derive(Hash, Eq, PartialEq, Copy, Clone, Debug)]
enum BlockType {
    Air,
    Water,
    Grass,
    Dirt,
    Stone,
    Log,
    Leaves,
    Rose,
    Dandelion,
    Bedrock,
}

impl BlockType {
    fn is_non_transparent(block: BlockType) -> bool {
        match block {
            BlockType::Air => false,
            BlockType::Water => false,
            BlockType::Leaves => false,
            BlockType::Rose => false,
            BlockType::Dandelion => false,
            _ => true
        }
    }
}

#[derive(Debug)]
struct ParseBlockTypeError {
    invalid_block: String
}

impl std::fmt::Display for ParseBlockTypeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Invalid block type given: {}", self.invalid_block)
    }
}

impl std::str::FromStr for BlockType {
    type Err = ParseBlockTypeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "air" => Ok(Self::Air),
            "water" => Ok(Self::Water),
            "grass" => Ok(Self::Grass),
            "dirt" => Ok(Self::Dirt),
            "stone" => Ok(Self::Stone),
            "log" => Ok(Self::Log),
            "leaves" => Ok(Self::Leaves),
            "rose" => Ok(Self::Rose),
            "dandelion" => Ok(Self::Dandelion),
            "bedrock" => Ok(Self::Bedrock),
            _ => Err(ParseBlockTypeError { invalid_block: s.to_string() })
        }
    }
}

struct Block {
    block_type: BlockType,
}

struct World {
    blocks: Vec<Vec<Vec<Block>>>, // 3D world
    textures: HashMap<BlockType, Texture2D>,
    visible_blocks: Vec<(f32, f32, BlockType)>,
    last_display_offset: Vec3,
}

impl World {
    fn new(textures: HashMap<BlockType, Texture2D>) -> Self {
        let perlin = Perlin::new(rand::gen_range(0, u32::MAX));
        let mut height_map = vec![];
        for x in 0..WORLD_SIZE_X {
            let mut column = vec![];
            for z in 0..WORLD_SIZE_Z {
                let mut row = vec![];
                let mut height = 0.0;
                for octave in 0..16 {
                    height += perlin.get([x as f64 / WORLD_SIZE_X as f64 * 2.0f64.powi(octave), z as f64 / WORLD_SIZE_Z as f64 * 2.0f64.powi(octave)]) * 2.0f64.powi(-octave) + MAGIC_NUMBER;
                }
                row.push(height);
                column.push(row);
            }
            height_map.push(column);
        }
        let mut caves = vec![];
        for x in 0..WORLD_SIZE_X {
            let mut column = vec![];
            for y in 0..WORLD_SIZE_Y {
                let mut row = vec![];
                for z in 0..WORLD_SIZE_Z {
                    let cave = perlin.get([x as f64 / WORLD_SIZE_X as f64 * ANOTHER_MAGIC_NUMBER, y as f64 / WORLD_SIZE_Y as f64 * ANOTHER_MAGIC_NUMBER, z as f64 / WORLD_SIZE_Z as f64 * 4.0]) * ANOTHER_MAGIC_NUMBER;
                    row.push(cave);
                }
                column.push(row);
            }
            caves.push(column);
        }
        let mut blocks = vec![];
        // X = north/south, Y = up/down, Z = east/west
        for x in 0..WORLD_SIZE_X {
            let mut column = vec![];
            for y in 0..WORLD_SIZE_Y {
                let mut row = vec![];
                for z in 0..WORLD_SIZE_Z {
                    if y == 0 {
                        row.push(Block { block_type: BlockType::Bedrock });
                    } else if caves[x][y][z] > 0.65 {
                        if y < 4 {
                            row.push(Block { block_type: BlockType::Water });
                        } else {
                            row.push(Block { block_type: BlockType::Air });
                        }
                    } else if y == 1 && rand::gen_range(0, 3) == 0 {
                        row.push(Block { block_type: BlockType::Bedrock });
                    } else if (y as f64) < height_map[x][z][0] * 16.0 {
                        if (y as f64) < height_map[x][z][0] * 16.0 - 3.0 {
                            row.push(Block { block_type: BlockType::Stone });
                        } else if (y as f64) < height_map[x][z][0] * 16.0 - 1.0 {
                            row.push(Block { block_type: BlockType::Dirt });
                        } else {
                            row.push(Block { block_type: BlockType::Grass });
                        }
                    } else {
                        if y < 4 {
                            row.push(Block { block_type: BlockType::Water });
                        } else {
                            row.push(Block { block_type: BlockType::Air });
                        }
                    }
                }
                column.push(row);
            }
            blocks.push(column);
        }
        fn place_structure(blocks: &mut Vec<Vec<Vec<Block>>>, x: usize, y: usize, z: usize, structure: Vec<Vec<Vec<BlockType>>>) {
            for dx in 0..structure.len() {
                for dy in 0..structure[0].len() {
                    for dz in 0..structure[0][0].len() {
                        if x + dx < WORLD_SIZE_X && y + dy < WORLD_SIZE_Y && z + dz < WORLD_SIZE_Z && structure[dx][dy][dz] != BlockType::Air {
                            blocks[x + dx][y + dy][z + dz] = Block { block_type: structure[dx][dy][dz] };
                        }
                    }
                }
            }
        }
        let tree = vec![
            vec![
                vec!["air", "air", "air", "air", "air"],
                vec!["air", "air", "air", "air", "air"],
                vec!["air", "leaves", "leaves", "leaves", "leaves"],
                vec!["leaves", "leaves", "leaves", "leaves", "leaves"],
                vec!["air", "air", "air", "air", "air"],
                vec!["air", "air", "air", "air", "air"],
                vec!["air", "air", "air", "air", "air"],
            ],
            vec![
                vec!["air", "air", "leaves", "air", "air"],
                vec!["air", "air", "leaves", "air", "air"],
                vec!["leaves", "leaves", "leaves", "leaves", "leaves"],
                vec!["leaves", "leaves", "leaves", "leaves", "leaves"],
                vec!["air", "air", "air", "air", "air"],
                vec!["air", "air", "air", "air", "air"],
                vec!["air", "air", "air", "air", "air"],
            ],
            vec![
                vec!["air", "leaves", "leaves", "leaves", "air"],
                vec!["air", "leaves", "leaves", "leaves", "air"],
                vec!["leaves", "leaves", "log", "leaves", "leaves"],
                vec!["leaves", "leaves", "log", "leaves", "leaves"],
                vec!["air", "air", "log", "air", "air"],
                vec!["air", "air", "log", "air", "air"],
                vec!["air", "air", "log", "air", "air"],
            ],
            vec![
                vec!["air", "air", "leaves", "air", "air"],
                vec!["air", "air", "leaves", "air", "air"],
                vec!["leaves", "leaves", "leaves", "leaves", "leaves"],
                vec!["leaves", "leaves", "leaves", "leaves", "leaves"],
                vec!["air", "air", "air", "air", "air"],
                vec!["air", "air", "air", "air", "air"],
                vec!["air", "air", "air", "air", "air"],
            ],
            vec![
                vec!["air", "air", "air", "air", "air"],
                vec!["air", "air", "air", "air", "air"],
                vec!["air", "leaves", "leaves", "leaves", "leaves"],
                vec!["air", "leaves", "leaves", "leaves", "leaves"],
                vec!["air", "air", "air", "air", "air"],
                vec!["air", "air", "air", "air", "air"],
                vec!["air", "air", "air", "air", "air"],
            ],
        ];
        let tree: Vec<Vec<Vec<BlockType>>> = tree.iter().map(|row| {
            row.iter().rev().map(|row| {
                row.iter().rev().map(|block| {
                    block.parse().unwrap()
                }).collect()
            }).collect()
        }).collect();
        
        for x in 0..WORLD_SIZE_X {
            for z in 0..WORLD_SIZE_Z {
                if blocks[x][((height_map[x][z][0] * WORLD_SIZE_Y as f64 / 4.0) as usize).min(WORLD_SIZE_Y - 1)][z].block_type == BlockType::Grass &&
                    blocks[x][((height_map[x][z][0] * WORLD_SIZE_Y as f64 / 4.0 + 1.0) as usize).min(63)][WORLD_SIZE_Y - 1].block_type == BlockType::Air {
                    if rand::gen_range(0, 100) == 0 {
                        place_structure(&mut blocks, x, ((height_map[x][z][0] * WORLD_SIZE_Y as f64 / 4.0) as usize).min(WORLD_SIZE_Y - 1), z, tree.clone());
                    }
                }
            }
        }
        
        let mut flower_patches = vec![];
        for x in 0..WORLD_SIZE_X {
            for z in 0..WORLD_SIZE_Z {
                // Perlin noise to determine if there should be a flower patch
                let noise = perlin.get([x as f64 / WORLD_SIZE_X as f64 * ANOTHER_MAGIC_NUMBER, z as f64 / WORLD_SIZE_Z as f64 * ANOTHER_MAGIC_NUMBER]);
                if noise > 0.5 {
                    flower_patches.push((x, z));
                }
            }
        }

        for x in 0..64 {
            for z in 0..64 {
                if blocks[x][((height_map[x][z][0] * WORLD_SIZE_Y as f64 / 4.0) as usize).min(WORLD_SIZE_Y - 1)][z].block_type == BlockType::Grass {
                    if flower_patches.contains(&(x, z)) && blocks[x][((height_map[x][z][0] * WORLD_SIZE_Y as f64 / 4.0 + 1.0) as usize).min(WORLD_SIZE_Y - 1)][z].block_type == BlockType::Air {
                        if rand::gen_range(0, 2) == 0 {
                            blocks[x][((height_map[x][z][0] * WORLD_SIZE_Y as f64 / 4.0 + 1.0) as usize).min(WORLD_SIZE_Y - 1)][z] = Block { block_type: BlockType::Rose };
                        } else {
                            blocks[x][((height_map[x][z][0] * WORLD_SIZE_Y as f64 / 4.0 + 1.0) as usize).min(WORLD_SIZE_Y - 1)][z] = Block { block_type: BlockType::Dandelion };
                        }
                    }
                }
            }
        }

        World { blocks, textures, visible_blocks: vec![], last_display_offset: vec3(0.0, 0.0, 0.0) }
    }

    fn update_visibility(&mut self, display_offset: Vec3) { // optimizing this is so freaking hard
        self.visible_blocks.clear();

        // Decide chunk size based on window size
        let chunk_x = ((screen_width() / BLOCK_FULL_SIZE / 7.0) as usize).min(WORLD_SIZE_X);
        let chunk_z = ((screen_height() / BLOCK_FULL_SIZE / 7.0) as usize).min(WORLD_SIZE_Z);

        let cam_x_min = (display_offset.x - (chunk_x as f32 / 2.0) * BLOCK_FULL_SIZE).max(0.0) as usize;
        let cam_x_max = (display_offset.x + (chunk_x as f32 / 2.0) * BLOCK_FULL_SIZE).min(WORLD_SIZE_X as f32) as usize;
        let cam_z_min = (display_offset.z - (chunk_z as f32 / 2.0) * BLOCK_FULL_SIZE).max(0.0) as usize;
        let cam_z_max = (display_offset.z + (chunk_z as f32 / 2.0) * BLOCK_FULL_SIZE).min(WORLD_SIZE_Z as f32) as usize;

        for x in cam_x_min..cam_x_max {
            for y in 0..WORLD_SIZE_Y {
                for z in cam_z_min..cam_z_max {
                    let screen_x = ((x as f32 - display_offset.x) - (z as f32 - display_offset.z)) * BLOCK_FULL_SIZE / 2.0;
                    let screen_y = ((x as f32 - display_offset.x) + (z as f32 - display_offset.z)) * BLOCK_FULL_SIZE / 4.0 - (y as f32 - display_offset.y) * BLOCK_FULL_SIZE / 2.0;

                    if screen_x < -BLOCK_FULL_SIZE * 2.0 ||
                       screen_x > screen_width() + BLOCK_FULL_SIZE * 2.0 ||
                       screen_y < -BLOCK_FULL_SIZE * 2.0 ||
                       screen_y > screen_height() + BLOCK_FULL_SIZE * 2.0 {
                        continue;
                    }

                    let mut is_hidden = false;
                    let mut i = 1;
                    while i < 5 {
                        let nx = x + i;
                        let ny = y + i;
                        let nz = z + i;
                        if nx >= WORLD_SIZE_X || ny >= WORLD_SIZE_Y || nz >= WORLD_SIZE_Z {
                            break;
                        }
                        if BlockType::is_non_transparent(self.blocks[nx][ny][nz].block_type) {
                            is_hidden = true;
                            break;
                        }
                        i += 1;
                    }

                    if is_hidden {
                        continue;
                    }

                    let block = &self.blocks[x][y][z];
                    if block.block_type != BlockType::Air {
                        self.visible_blocks.push((screen_x, screen_y, block.block_type));
                    }
                }
            }
        }
    }

    fn update_visibility_if_moved(&mut self, display_offset: Vec3) {
        let display_offset_rounded = vec3((display_offset.x * 10.0).round() / 10.0, (display_offset.y * 10.0).round() / 10.0, (display_offset.z * 10.0).round() / 10.0);
        let last_display_offset_rounded = vec3((self.last_display_offset.x * 10.0).round() / 10.0, (self.last_display_offset.y * 10.0).round() / 10.0, (self.last_display_offset.z * 10.0).round() / 10.0);
        if display_offset_rounded == last_display_offset_rounded {
            return; // No movement, no need to update
        }

        self.last_display_offset = display_offset; // Store the new position

        self.update_visibility(display_offset);
    }

    fn draw(&self, display_offset: Vec3) {
        for (screen_x, screen_y, block_type) in &self.visible_blocks {
            if let Some(texture) = self.textures.get(&block_type) {
                draw_texture_ex(texture, screen_x + display_offset.x, screen_y + display_offset.y, WHITE, DrawTextureParams {
                    dest_size: Some(vec2(BLOCK_FULL_SIZE, BLOCK_FULL_SIZE)),
                    ..Default::default()
                });
            } else {
                draw_rectangle(screen_x + display_offset.x, screen_y + display_offset.y, BLOCK_FULL_SIZE, BLOCK_FULL_SIZE, PINK);
            }
        }
    }
}

fn load_ase_file_into_texture(path: &str) -> Texture2D {
    let ase = AsepriteFile::read_file(Path::new(path)).unwrap();
    let image = ase.frame(0).image();
    let vec = image.clone().into_raw();
    let texture = Texture2D::from_rgba8(image.width() as u16, image.height() as u16, &vec);
    texture.set_filter(FilterMode::Nearest);
    texture
}

fn window_conf() -> Conf {
    Conf {
        window_title: "Isometric Minecraft".to_owned(),
        window_width: 1280,
        window_height: 720,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let water_texture = load_ase_file_into_texture("assets/water.ase");
    let grass_texture = load_ase_file_into_texture("assets/grass.ase");
    let dirt_texture = load_ase_file_into_texture("assets/dirt.ase");
    let stone_texture = load_ase_file_into_texture("assets/stone.ase");
    let log_texture = load_ase_file_into_texture("assets/log.ase");
    let leaves_texture = load_ase_file_into_texture("assets/leaves.ase");
    let rose_texture = load_ase_file_into_texture("assets/rose.ase");
    let dandelion_texture = load_ase_file_into_texture("assets/dandelion.ase");
    let bedrock_texture = load_ase_file_into_texture("assets/bedrock.ase");
    let mut textures = HashMap::new();
    textures.insert(BlockType::Water, water_texture);
    textures.insert(BlockType::Grass, grass_texture);
    textures.insert(BlockType::Dirt, dirt_texture);
    textures.insert(BlockType::Stone, stone_texture);
    textures.insert(BlockType::Log, log_texture);
    textures.insert(BlockType::Leaves, leaves_texture);
    textures.insert(BlockType::Rose, rose_texture);
    textures.insert(BlockType::Dandelion, dandelion_texture);
    textures.insert(BlockType::Bedrock, bedrock_texture);
    rand::srand(std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs() as u64);
    let mut offset = vec2(-20.0, -4.0);
    let mut offset_vel = vec2(0.0, 0.0);
    let mut start = std::time::Instant::now();
    let mut world = World::new(textures);
    // world.update_visible_blocks(vec3(offset.x, 0.0, offset.y)); 
    world.update_visibility_if_moved(vec3(offset.x, 0.1, offset.y)); // Hacky fix for the first frame not rendering
    let world_gen_time = start.elapsed();
    println!("    World generation took {} micros ({} seconds)", world_gen_time.as_micros(), world_gen_time.as_secs_f64());

    loop {
        clear_background(BLACK);
        if is_key_down(KeyCode::W) {
            offset_vel.y -= 1.0;
        }
        if is_key_down(KeyCode::S) {
            offset_vel.y += 1.0;
        }
        if is_key_down(KeyCode::A) {
            offset_vel.x -= 1.0;
        }
        if is_key_down(KeyCode::D) {
            offset_vel.x += 1.0;
        }

        offset += offset_vel * get_frame_time() * 2.0;
        offset_vel *= 0.9;
        start = std::time::Instant::now();
        // world.update_visible_blocks(vec3(offset.x, 0.0, offset.y));
        world.update_visibility_if_moved(vec3(offset.x, 0.0, offset.y));
        let update_visibility_time = start.elapsed();
        print!("    Updating took {} micros ({} seconds)  |  ", update_visibility_time.as_micros(), update_visibility_time.as_secs_f64());
        start = std::time::Instant::now();
        world.draw(vec3(offset.x, 0.0, offset.y));
        let draw_time = start.elapsed();
        print!("Drawing took {} micros ({} seconds)     \r", draw_time.as_micros(), draw_time.as_secs_f64());
        std::io::stdout().flush().unwrap();
        draw_text(format!("FPS: {}", get_fps()).as_str(), 10.0, 50.0, 48.0, WHITE);
        draw_text(format!("Took {} milliseconds to generate world", world_gen_time.as_millis()).as_str(), 10.0, screen_height() - 40.0, 32.0, WHITE);
        draw_text(format!("Took {} milliseconds to update visibility", update_visibility_time.as_millis()).as_str(), 10.0, screen_height() - 80.0, 32.0, WHITE);
        draw_text(format!("Took {} milliseconds to draw", draw_time.as_millis()).as_str(), 10.0, screen_height() - 120.0, 32.0, WHITE);
        next_frame().await
    }
}
