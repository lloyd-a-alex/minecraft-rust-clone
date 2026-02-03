# 🎮 LEGACY MINECRAFT CLONE - IMPLEMENTATION ROADMAP v2.0

**Critical Issues, Missing Features & New Architectural Additions**

---

## CRITICAL BUG FIX #1: FACE CULLING LOGIC IS BACKWARDS

### The Problem (Current Code)

```rust
// renderer.rs line ~80-95
if !world.get_block(BlockPos { x: worldpos.x, y: worldpos.y + 1, z: worldpos.z }).is_solid()
    self.add_face(/* top */);
```

**This is WRONG.** You're checking:
- "If the block ABOVE is NOT solid, render top face"

**This should be:**
- "If the block ABOVE is AIR/WATER (transparent), render top face"

The difference:
- `is_solid()` returns `!= Air && != Water`
- But LEAVES blocks return `is_solid() = true` even though they're visually semi-transparent
- You need to check `is_transparent()` instead

### Solution #1A: Fix BlockType Methods (IMMEDIATE)

**File: `world.rs` - REPLACE this function:**

```rust
impl BlockType {
  pub fn is_solid(&self) -> bool {
    !matches!(self, BlockType::Air | BlockType::Water)
  }

  pub fn is_transparent(&self) -> bool {
    matches!(self, BlockType::Air | BlockType::Leaves | BlockType::Water)
  }

  pub fn should_cull_face(&self) -> bool {
    // Returns TRUE if this block occludes adjacent faces
    matches!(
      self,
      BlockType::Grass
        | BlockType::Dirt
        | BlockType::Stone
        | BlockType::Wood
        | BlockType::Snow
        | BlockType::Sand
        | BlockType::Bedrock
    )
  }
}
```

**File: `renderer.rs` - REPLACE the culling logic (~line 80-100):**

```rust
// CORRECT FACE CULLING
let above = world.get_block(BlockPos {
  x: worldpos.x,
  y: worldpos.y + 1,
  z: worldpos.z,
});
if above.is_transparent() {
  self.add_face(/* top */);
}

let below = world.get_block(BlockPos {
  x: worldpos.x,
  y: worldpos.y - 1,
  z: worldpos.z,
});
if below.is_transparent() {
  self.add_face(/* bottom */);
}

// ... repeat for all 6 directions
```

**Why this matters:** 
- Leaves are `is_solid()` but `is_transparent()` 
- Your code skips rendering faces next to leaves (looks wrong)
- Correct logic: only skip faces when block is OPAQUE (solid + not transparent)

---

## CRITICAL BUG FIX #2: VERTEX WINDING ORDER (NORMAL DIRECTION)

### The Problem

Some faces may be rendered with **backwards normals**, causing lighting to be inverted or backface culling issues.

### Solution #2: Fix Vertex Order for ALL Faces

**File: `renderer.rs` - `add_face()` function - ALL 6 FACES:**

```rust
fn add_face(&mut self, vertices: &mut Vec<Vertex>, indices: &mut Vec<u16>, 
    index_offset: &mut u16, pos: BlockPos, face_id: usize, tex_id: u32) {

  let x = pos.x as f32;
  let y = pos.y as f32;
  let z = pos.z as f32;

  // CRITICAL: Vertex order must be COUNTER-CLOCKWISE when viewed from OUTSIDE the block
  // This ensures normals point outward for correct lighting & culling

  let (v0, v1, v2, v3) = match face_id {
    // TOP (+Y) - viewed from above (camera looking down)
    0 => (
      Vertex { position: [x, y + 1.0, z], texcoords: [0.0, 0.0], ao: 1.0, texindex: tex_id },
      Vertex { position: [x + 1.0, y + 1.0, z], texcoords: [1.0, 0.0], ao: 1.0, texindex: tex_id },
      Vertex { position: [x + 1.0, y + 1.0, z + 1.0], texcoords: [1.0, 1.0], ao: 1.0, texindex: tex_id },
      Vertex { position: [x, y + 1.0, z + 1.0], texcoords: [0.0, 1.0], ao: 1.0, texindex: tex_id },
    ),

    // BOTTOM (-Y) - viewed from below (camera looking up)
    1 => (
      Vertex { position: [x, y, z + 1.0], texcoords: [0.0, 0.0], ao: 0.5, texindex: tex_id },
      Vertex { position: [x + 1.0, y, z + 1.0], texcoords: [1.0, 0.0], ao: 0.5, texindex: tex_id },
      Vertex { position: [x + 1.0, y, z], texcoords: [1.0, 1.0], ao: 0.5, texindex: tex_id },
      Vertex { position: [x, y, z], texcoords: [0.0, 1.0], ao: 0.5, texindex: tex_id },
    ),

    // LEFT (-X) - viewed from the left (camera looking right)
    2 => (
      Vertex { position: [x, y, z], texcoords: [0.0, 1.0], ao: 0.8, texindex: tex_id },
      Vertex { position: [x, y + 1.0, z], texcoords: [0.0, 0.0], ao: 0.8, texindex: tex_id },
      Vertex { position: [x, y + 1.0, z + 1.0], texcoords: [1.0, 0.0], ao: 0.8, texindex: tex_id },
      Vertex { position: [x, y, z + 1.0], texcoords: [1.0, 1.0], ao: 0.8, texindex: tex_id },
    ),

    // RIGHT (+X) - viewed from the right (camera looking left)
    3 => (
      Vertex { position: [x + 1.0, y, z + 1.0], texcoords: [0.0, 1.0], ao: 0.8, texindex: tex_id },
      Vertex { position: [x + 1.0, y + 1.0, z + 1.0], texcoords: [0.0, 0.0], ao: 0.8, texindex: tex_id },
      Vertex { position: [x + 1.0, y + 1.0, z], texcoords: [1.0, 0.0], ao: 0.8, texindex: tex_id },
      Vertex { position: [x + 1.0, y, z], texcoords: [1.0, 1.0], ao: 0.8, texindex: tex_id },
    ),

    // FRONT (+Z) - viewed from the front (camera looking in -Z)
    4 => (
      Vertex { position: [x + 1.0, y, z + 1.0], texcoords: [0.0, 1.0], ao: 0.6, texindex: tex_id },
      Vertex { position: [x + 1.0, y + 1.0, z + 1.0], texcoords: [0.0, 0.0], ao: 0.6, texindex: tex_id },
      Vertex { position: [x, y + 1.0, z + 1.0], texcoords: [1.0, 0.0], ao: 0.6, texindex: tex_id },
      Vertex { position: [x, y, z + 1.0], texcoords: [1.0, 1.0], ao: 0.6, texindex: tex_id },
    ),

    // BACK (-Z) - viewed from the back (camera looking in +Z)
    5 => (
      Vertex { position: [x, y, z], texcoords: [0.0, 1.0], ao: 0.6, texindex: tex_id },
      Vertex { position: [x, y + 1.0, z], texcoords: [0.0, 0.0], ao: 0.6, texindex: tex_id },
      Vertex { position: [x + 1.0, y + 1.0, z], texcoords: [1.0, 0.0], ao: 0.6, texindex: tex_id },
      Vertex { position: [x + 1.0, y, z], texcoords: [1.0, 1.0], ao: 0.6, texindex: tex_id },
    ),

    _ => panic!("Invalid face ID"),
  };

  vertices.push(v0);
  vertices.push(v1);
  vertices.push(v2);
  vertices.push(v3);

  // Two triangles per quad: (0,1,2) and (0,2,3)
  indices.push(index_offset);
  indices.push(index_offset + 1);
  indices.push(index_offset + 2);

  indices.push(index_offset);
  indices.push(index_offset + 2);
  indices.push(index_offset + 3);

  *index_offset += 4;
}
```

---

## CRITICAL BUG FIX #3: CHUNK BOUNDARY ISSUES

### The Problem

When checking adjacent blocks for culling, if the adjacent block is in a different chunk, `world.get_block()` may return `Air` incorrectly if that chunk hasn't loaded yet.

### Solution #3: Safe Boundary Checking

**File: `renderer.rs` - in `rebuild_chunks()` function:**

```rust
pub fn rebuild_chunks(&mut self, world: &World) {
  self.chunk_meshes.clear();

  for (chunk_key, chunk) in &world.chunks {
    let (cx, cz) = chunk_key;
    let mut vertices = Vec::new();
    let mut indices = Vec::new();
    let mut index_offset: u16 = 0;

    let chunk_x_world = cx * CHUNK_SIZE_X as i32;
    let chunk_z_world = cz * CHUNK_SIZE_Z as i32;

    for x in 0..CHUNK_SIZE_X {
      for y in 0..CHUNK_SIZE_Y {
        for z in 0..CHUNK_SIZE_Z {
          let block = chunk.get_block(x, y, z);
          if !block.is_solid() {
            continue;
          }

          let world_pos = BlockPos {
            x: chunk_x_world + x as i32,
            y: y as i32,
            z: chunk_z_world + z as i32,
          };

          let (tex_top, tex_bottom, tex_side) = block.get_texture_indices();

          // ===== SAFE CULLING WITH BOUNDARY CHECKS =====

          // TOP (+Y)
          let above = if y + 1 < CHUNK_SIZE_Y {
            chunk.get_block(x, y + 1, z)
          } else {
            world.get_block(BlockPos {
              x: world_pos.x,
              y: world_pos.y + 1,
              z: world_pos.z,
            })
          };
          if above.is_transparent() {
            self.add_face(&mut vertices, &mut indices, &mut index_offset, world_pos, 0, tex_top);
          }

          // BOTTOM (-Y)
          let below = if y > 0 {
            chunk.get_block(x, y - 1, z)
          } else {
            world.get_block(BlockPos {
              x: world_pos.x,
              y: world_pos.y - 1,
              z: world_pos.z,
            })
          };
          if below.is_transparent() {
            self.add_face(&mut vertices, &mut indices, &mut index_offset, world_pos, 1, tex_bottom);
          }

          // LEFT (-X)
          let left = if x > 0 {
            chunk.get_block(x - 1, y, z)
          } else {
            world.get_block(BlockPos {
              x: world_pos.x - 1,
              y: world_pos.y,
              z: world_pos.z,
            })
          };
          if left.is_transparent() {
            self.add_face(&mut vertices, &mut indices, &mut index_offset, world_pos, 2, tex_side);
          }

          // RIGHT (+X)
          let right = if x + 1 < CHUNK_SIZE_X {
            chunk.get_block(x + 1, y, z)
          } else {
            world.get_block(BlockPos {
              x: world_pos.x + 1,
              y: world_pos.y,
              z: world_pos.z,
            })
          };
          if right.is_transparent() {
            self.add_face(&mut vertices, &mut indices, &mut index_offset, world_pos, 3, tex_side);
          }

          // BACK (-Z)
          let back = if z > 0 {
            chunk.get_block(x, y, z - 1)
          } else {
            world.get_block(BlockPos {
              x: world_pos.x,
              y: world_pos.y,
              z: world_pos.z - 1,
            })
          };
          if back.is_transparent() {
            self.add_face(&mut vertices, &mut indices, &mut index_offset, world_pos, 5, tex_side);
          }

          // FRONT (+Z)
          let front = if z + 1 < CHUNK_SIZE_Z {
            chunk.get_block(x, y, z + 1)
          } else {
            world.get_block(BlockPos {
              x: world_pos.x,
              y: world_pos.y,
              z: world_pos.z + 1,
            })
          };
          if front.is_transparent() {
            self.add_face(&mut vertices, &mut indices, &mut index_offset, world_pos, 4, tex_side);
          }
        }
      }
    }

    if !vertices.is_empty() {
      let vertex_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Chunk VB"),
        contents: bytemuck::cast_slice(&vertices),
        usage: wgpu::BufferUsages::VERTEX,
      });

      let index_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Chunk IB"),
        contents: bytemuck::cast_slice(&indices),
        usage: wgpu::BufferUsages::INDEX,
      });

      self.chunk_meshes.push(ChunkMesh {
        vertex_buffer,
        index_buffer,
        index_count: indices.len() as u32,
      });
    }
  }
}
```

---

## PHASE 2: MISSING FEATURES FROM MANIFESTO

### Feature #1: Smooth Lighting (Per-Vertex Ambient Occlusion)

**Status:** NOT IMPLEMENTED  
**Priority:** HIGH  
**Manifesto Section:** 1.2.1

**Implementation Plan:**

1. **Add Light Data to Vertex Structure** (world.rs)
   ```rust
   pub struct Vertex {
     position: [f32; 3],
     texcoords: [f32; 2],
     ao: f32,                    // ← CURRENTLY HERE BUT NOT USED
     texindex: u32,
     light: u32,                 // ← ADD THIS (sky_light:4bits | block_light:4bits)
     normal: [i8; 3],            // ← ADD THIS (face normal for lighting)
   }
   ```

2. **Implement Light Propagation** (new file: `lighting.rs`)
   - Skylight: flood-fill from top downward (level 15 → 0)
   - Block light: (optional for now) hardcode to 15
   - Per-vertex averaging from 4 adjacent blocks

3. **Pass Light to Fragment Shader**
   - Unpack light value in vertex shader
   - Interpolate across face
   - Apply AO multiplier in fragment shader

**Estimated Effort:** 800 lines  
**Blocking:** Shader updates + vertex struct changes

---

### Feature #2: Day-Night Cycle & Sky Color Interpolation

**Status:** PARTIALLY IMPLEMENTED (skeleton in renderer)  
**Priority:** HIGH  
**Manifesto Section:** 1.2.2

**Current Code Problem:**
```rust
let cycle = elapsed * 0.05.sin(); // This cycles too fast (every ~2 seconds)
```

**Correct Implementation:**
```rust
// 20-minute Minecraft day = 24000 ticks
// Let's do 2-minute cycle for testing: 120 seconds
const CYCLE_DURATION_SECS: f32 = 120.0;

let time_phase = (elapsed % CYCLE_DURATION_SECS) / CYCLE_DURATION_SECS; // [0, 1]
let angle = time_phase * std::f32::consts::PI * 2.0;

let sky_color = match time_phase {
  t if t < 0.25 => {
    // Dawn: lerp from (25,25,80) to (135,206,235)
    let blend = (t / 0.25) as f32;
    lerp_color([25, 25, 80], [135, 206, 235], blend)
  },
  t if t < 0.5 => {
    // Noon: stay bright
    [135, 206, 235]
  },
  t if t < 0.75 => {
    // Dusk: lerp to (255,140,60)
    let blend = ((t - 0.5) / 0.25) as f32;
    lerp_color([135, 206, 235], [255, 140, 60], blend)
  },
  _ => {
    // Night: lerp back to (25,25,80)
    let blend = ((time_phase - 0.75) / 0.25) as f32;
    lerp_color([255, 140, 60], [25, 25, 80], blend)
  },
};

let brightness = 0.3 + 0.7 * (angle.sin() * 0.5 + 0.5); // [0.3, 1.0]
```

**Estimated Effort:** 200 lines  
**Blocking:** None (shader already has time_uniform)

---

### Feature #3: Greedy Meshing (Geometry Optimization)

**Status:** NOT IMPLEMENTED  
**Priority:** MEDIUM (nice-to-have, not blocking)  
**Manifesto Section:** 1.1.3

**Current approach:** Naive cube rendering (24 vertices per block face)  
**Potential:** Greedy meshing reduces to ~4-8 vertices per merged quad

**For now:** Skip greedy meshing. Works fine with current geometry count.

**When to implement:** If render time > 5ms per chunk

---

### Feature #4: Frustum Culling

**Status:** NOT IMPLEMENTED  
**Priority:** MEDIUM  
**Manifesto Section:** 1.3.3

**Current Issue:** Rendering ALL chunks regardless of view distance

**Quick Win (16-chunk render distance):**
```rust
fn should_render_chunk(chunk_pos: (i32, i32), camera_pos: [f32; 3]) -> bool {
  let cx = chunk_pos.0 as f32;
  let cz = chunk_pos.1 as f32;
  let dist_x = (cx * 16.0 - camera_pos[0]).abs();
  let dist_z = (cz * 16.0 - camera_pos[2]).abs();
  dist_x < 256.0 && dist_z < 256.0 // 16 chunks = 256 blocks
}
```

**Full Implementation:** Extract frustum planes from proj×view matrix  
**Estimated Effort:** 300 lines

---

## PHASE 3: NEW FEATURES

### New Feature #1: Proper Camera System

**Status:** BASIC IMPLEMENTATION (needs improvements)  
**Manifesto Section:** 1.4

**Missing:**
- Proper perspective matrix (currently hacked)
- Correct view matrix calculation
- Field-of-view (should be 60°)

**Quick Fix (player.rs):**
```rust
pub fn build_view_projection_matrix(&self, width: f32, height: f32) -> [[f32; 4]; 4] {
  let aspect = width / height;
  let fov = 60.0_f32.to_radians(); // 60° FOV (Minecraft default)
  let near = 0.1;
  let far = 300.0;

  let f = 1.0 / (fov / 2.0).tan();
  let nf = 1.0 / (near - far);

  let proj = [
    [f / aspect, 0.0, 0.0, 0.0],
    [0.0, f, 0.0, 0.0],
    [0.0, 0.0, (far + near) * nf, -1.0],
    [0.0, 0.0, 2.0 * far * near * nf, 0.0],
  ];

  let view = self.get_view_matrix();

  // Matrix multiply: proj × view
  matrix_multiply_4x4(proj, view)
}
```

---

### New Feature #2: Block Breaking/Placing (Creative Mode)

**Status:** NOT IMPLEMENTED  
**Manifesto Section:** N/A (not in manifesto but essential)

**Requirements:**
- Left-click: place block (selected from hotbar)
- Right-click: break block
- Raycast from camera through world
- Update chunk mesh on block change

**Raycast Implementation:**
```rust
pub fn raycast(&self, world: &World, max_distance: f32) -> Option<BlockPos> {
  const STEP_SIZE: f32 = 0.1;
  let mut distance = 0.0;

  while distance < max_distance {
    let ray_point = self.position + self.direction * distance;
    let block_pos = BlockPos {
      x: ray_point[0].floor() as i32,
      y: ray_point[1].floor() as i32,
      z: ray_point[2].floor() as i32,
    };

    let block = world.get_block(block_pos);
    if block.is_solid() {
      return Some(block_pos);
    }

    distance += STEP_SIZE;
  }
  None
}
```

---

### New Feature #3: Proper Texture Filtering (Mipmap Fix)

**Status:** BROKEN (nearest neighbor causes artifacts)  
**Manifesto Section:** 1.3.2

**Current Code (renderer.rs):**
```rust
let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
  mag_filter: wgpu::FilterMode::Nearest,  // ← WRONG (causes artifacts)
  min_filter: wgpu::FilterMode::Nearest,  // ← WRONG
  mipmap_filter: wgpu::FilterMode::Nearest,
  ..Default::default()
});
```

**Correct Implementation:**
```rust
let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
  mag_filter: wgpu::FilterMode::Linear,   // Smooth close-up
  min_filter: wgpu::FilterMode::Linear,   // Smooth distance
  mipmap_filter: wgpu::FilterMode::Linear,
  address_mode_u: wgpu::AddressMode::ClampToEdge,
  address_mode_v: wgpu::AddressMode::ClampToEdge,
  ..Default::default()
});
```

**Why:** Prevents texture bleeding at distance while keeping sharp close-up detail

---

### New Feature #4: Water Rendering (Transparency + Wave Animation)

**Status:** NOT IMPLEMENTED  
**Manifesto Section:** N/A (advanced feature)

**Requirements:**
- Blend water as semi-transparent (alpha = 0.8)
- Render behind solid blocks (depth-test only)
- Optional: Animate UVs with time for wave effect

**Fragment Shader Changes:**
```wgsl
@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
  let base_color = textureSample(texture_atlas, sampler_linear, in.uv);

  if in.tex_index == 9u { // Water block
    // Water specific rendering
    let wave_offset = time_data.time * 2.0; // Oscillate over 2 seconds
    let wave_uv = in.uv + vec2<f32>(wave_offset, 0.0);
    let water_color = textureSample(texture_atlas, sampler_linear, wave_uv);
    return vec4<f32>(water_color.rgb * 0.8, 0.7); // 70% opacity
  }

  let lit_color = base_color.rgb * in.light_accum;
  return vec4<f32>(lit_color, base_color.a);
}
```

---

## PHASE 4: PERFORMANCE OPTIMIZATIONS

### Optimization #1: LOD (Level of Detail)

**Status:** NOT IMPLEMENTED  
**Impact:** 2-3× FPS improvement for far chunks

**Approach:**
- Load chunks at full detail near player (0-32 blocks)
- Reduce vertex count for far chunks (32-256 blocks)
- Skip rendering beyond 256 blocks

**Effort:** 400 lines

---

### Optimization #2: Batch Rendering

**Status:** SINGLE BATCH (inefficient)  
**Current:** Render each chunk separately  
**Better:** Batch chunks with same material

**Improvement:** Reduce draw calls from 16×16=256 to ~4-8

**Effort:** 200 lines

---

### Optimization #3: Async Chunk Generation

**Status:** SYNCHRONOUS (blocks main thread)  
**Problem:** Frame stutter when loading chunks

**Solution:** Use `tokio::spawn` for background chunk generation

**Effort:** 300 lines

---

## IMMEDIATE ACTION ITEMS (NEXT 2 HOURS)

### Priority 1: Fix Face Culling (CRITICAL)
1. [ ] Fix `is_transparent()` method in world.rs
2. [ ] Replace culling checks in renderer.rs `rebuild_chunks()`
3. [ ] Fix vertex winding order in `add_face()` for all 6 faces
4. [ ] Add boundary safety checks for chunk edges
5. [ ] Test: All 6 sides of blocks should render

**Expected Result:** Blocks render on all sides correctly

---

### Priority 2: Improve Lighting
1. [ ] Add light packing to Vertex struct (skip proper propagation for now)
2. [ ] Hardcode light value = 15.0 (full brightness)
3. [ ] Pass light to fragment shader
4. [ ] Apply AO multiplier from Vertex::ao field
5. [ ] Test: All blocks should render with consistent lighting

**Expected Result:** Better visual depth, proper block faces visible

---

### Priority 3: Implement Day-Night Cycle
1. [ ] Fix time calculation (120-second cycle for testing)
2. [ ] Implement piecewise color interpolation
3. [ ] Update sky colors in fragment shader
4. [ ] Test: Sky color should transition smoothly

**Expected Result:** Dynamic lighting matching Minecraft beta aesthetic

---

## SUMMARY TABLE

| Feature | Status | Effort | Priority | Impact |
|---------|--------|--------|----------|--------|
| Face Culling Fix | CRITICAL BUG | 1h | P0 | Blocks render all sides |
| Lighting System | Not done | 8h | P1 | Visual quality |
| Day-Night Cycle | Skeleton | 2h | P1 | Aesthetics |
| Smooth Lighting | Not done | 6h | P2 | Realism |
| Frustum Culling | Not done | 3h | P2 | Performance |
| Greedy Meshing | Not done | 10h | P3 | Performance |
| Block Breaking | Not done | 4h | P2 | Gameplay |
| Water Rendering | Not done | 4h | P3 | Visual quality |
| LOD System | Not done | 8h | P3 | FPS improvement |
| Async Loading | Not done | 6h | P3 | Smoothness |

---

## TESTING CHECKLIST

- [ ] All 6 faces of solid blocks render
- [ ] Leaves don't occlude adjacent faces
- [ ] Water blocks don't occlude adjacent faces
- [ ] Chunk boundaries work correctly
- [ ] Sky color transitions smoothly
- [ ] Lighting is consistent across faces
- [ ] No z-fighting or texture artifacts
- [ ] Performance > 60 FPS with 16-chunk render distance
- [ ] No memory leaks over 5 minutes gameplay

---

**Document Version:** 2.0  
**Last Updated:** 2026-01-29  
**Critical Blocker:** Face culling bug  
**Next Session Focus:** Fix bugs #1, #2, #3 then implement lighting  
