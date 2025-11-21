// ============================================
// KERNEL 1: Generate Vertices
// ============================================
// This shader generates vertices at the surface crossings in each cell
// by finding where the isosurface (value = 0) crosses cell edges.

// STEP 1: Define the bind group layout
// These match the Rust side BindGroupLayoutEntries in order (0, 1, 2, 3)
@group(0) @binding(0)
var<storage, read> density_field: array<f32>;  // Input scalar field

@group(0) @binding(1)
var<storage, read_write> vertices: array<f32>;  // Output vertex positions (x,y,z packed)

@group(0) @binding(2)
var<storage, read_write> vertex_valid: array<u32>;  // Output validity flags (1 = valid vertex)

@group(0) @binding(3)
var<uniform> dimensions: vec3<u32>;  // Grid dimensions (x, y, z)

// ===========================================================
// Helper function MUST be at global scope in WGSL
// ===========================================================
fn sample_density(x: u32, y: u32, z: u32) -> f32 {
    let index = x + y * dimensions.x + z * dimensions.x * dimensions.y;
    return density_field[index];
}

// STEP 2: Define workgroup size
// Must match the WORKGROUP_SIZE constant in Rust (8x8x8 = 512 threads per workgroup)
@compute @workgroup_size(8, 8, 8)
fn generate_vertices(
    @builtin(global_invocation_id) global_id: vec3<u32>,  // Unique thread ID across all workgroups
) {
    // STEP 3: Get cell coordinates
    // Each thread processes one cell in the 3D grid
    let cell_x = global_id.x;
    let cell_y = global_id.y;
    let cell_z = global_id.z;
    
    // STEP 4: Boundary check
    // We need to sample corners, so cells on the far edge can't form complete cells
    // A grid of size (10,10,10) has density values at positions 0-9
    // So cells exist from (0,0,0) to (8,8,8) - that's dimensions - 1
    if (cell_x >= dimensions.x - 1u || 
        cell_y >= dimensions.y - 1u || 
        cell_z >= dimensions.z - 1u) {
        return;  // This thread is outside the valid cell range
    }
    
    // STEP 5: Calculate flat index for this cell
    // Convert 3D cell position (x,y,z) to 1D array index
    // Formula: z * (width * height) + y * width + x
    let cell_index = cell_x + cell_y * dimensions.x + cell_z * dimensions.x * dimensions.y;
    
    // STEP 7: Define the 8 corners of this cell
    // A cell is a cube, and we need to check all 8 corners
    // Corner layout:
    //       7 -------- 6
    //      /|         /|
    //     / |        / |
    //    4 -------- 5  |
    //    |  3 ------|--2
    //    | /        | /
    //    |/         |/
    //    0 -------- 1
    var corners: array<vec3<u32>, 8>;
    corners[0] = vec3<u32>(cell_x,     cell_y,     cell_z);      // Bottom-front-left
    corners[1] = vec3<u32>(cell_x + 1u, cell_y,     cell_z);      // Bottom-front-right
    corners[2] = vec3<u32>(cell_x + 1u, cell_y + 1u, cell_z);      // Bottom-back-right
    corners[3] = vec3<u32>(cell_x,     cell_y + 1u, cell_z);      // Bottom-back-left
    corners[4] = vec3<u32>(cell_x,     cell_y,     cell_z + 1u);  // Top-front-left
    corners[5] = vec3<u32>(cell_x + 1u, cell_y,     cell_z + 1u);  // Top-front-right
    corners[6] = vec3<u32>(cell_x + 1u, cell_y + 1u, cell_z + 1u);  // Top-back-right
    corners[7] = vec3<u32>(cell_x,     cell_y + 1u, cell_z + 1u);  // Top-back-left
    
    // STEP 8: Define the 12 edges of the cell (as pairs of corner indices)
    // Each edge connects two corners where we'll check for surface crossings
    // Format: (start_corner_index, end_corner_index)
    var edges: array<vec2<u32>, 12>;
    edges[0]  = vec2<u32>(0u, 1u);   // Bottom front edge
    edges[1]  = vec2<u32>(1u, 2u);   // Bottom right edge
    edges[2]  = vec2<u32>(2u, 3u);   // Bottom back edge
    edges[3]  = vec2<u32>(3u, 0u);   // Bottom left edge
    edges[4]  = vec2<u32>(4u, 5u);   // Top front edge
    edges[5]  = vec2<u32>(5u, 6u);   // Top right edge
    edges[6]  = vec2<u32>(6u, 7u);   // Top back edge
    edges[7]  = vec2<u32>(7u, 4u);   // Top left edge
    edges[8]  = vec2<u32>(0u, 4u);   // Vertical front-left edge
    edges[9]  = vec2<u32>(1u, 5u);   // Vertical front-right edge
    edges[10] = vec2<u32>(2u, 6u);   // Vertical back-right edge
    edges[11] = vec2<u32>(3u, 7u);   // Vertical back-left edge
    
    // STEP 9: Find all edge crossings
    // Surface Nets works by averaging all the positions where the surface crosses edges
    var crossing_sum = vec3<f32>(0.0, 0.0, 0.0);  // Sum of all crossing positions
    var crossing_count = 0u;  // How many crossings we found
    
    // STEP 10: Check each edge for a surface crossing
    for (var i = 0u; i < 12u; i = i + 1u) {
        // Get the two corners that define this edge
        let corner_0_idx = edges[i].x;
        let corner_1_idx = edges[i].y;
        
        let p0 = corners[corner_0_idx];  // First corner position (grid coords)
        let p1 = corners[corner_1_idx];  // Second corner position (grid coords)
        
        // Sample the density at each corner
        let v0 = sample_density(p0.x, p0.y, p0.z);
        let v1 = sample_density(p1.x, p1.y, p1.z);
        
        // STEP 11: Check for sign change (surface crossing)
        // If v0 and v1 have different signs, the isosurface (value=0) crosses this edge
        // We use multiplication: if v0*v1 < 0, they have opposite signs
        if (v0 * v1 < 0.0) {
            // STEP 12: Linear interpolation to find exact crossing point
            // The surface is at value 0, so we interpolate between v0 and v1
            // Formula: t = v0 / (v0 - v1)
            // This gives us how far along the edge (0.0 to 1.0) the crossing occurs
            let t = v0 / (v0 - v1);
            
            // Interpolate position: crossing_point = p0 + t * (p1 - p0)
            // This gives us the exact 3D position where the surface crosses this edge
            let crossing_point = vec3<f32>(p0) + t * (vec3<f32>(p1) - vec3<f32>(p0));
            
            // Add this crossing to our sum
            crossing_sum = crossing_sum + crossing_point;
            crossing_count = crossing_count + 1u;
        }
    }
    
    // STEP 13: Create vertex if we found any crossings
    if (crossing_count > 0u) {
        // STEP 14: Average all crossing positions to get the vertex position
        // This is the key idea of Surface Nets - we place the vertex at the
        // average of all edge crossings in this cell
        let vertex_pos = crossing_sum / f32(crossing_count);
        
        // STEP 15: Store vertex in output buffer
        // Vertices are stored as flat array: [x0, y0, z0, x1, y1, z1, ...]
        // So vertex at cell_index goes at position cell_index * 3
        let vertex_base_index = cell_index * 3u;
        vertices[vertex_base_index + 0u] = vertex_pos.x;
        vertices[vertex_base_index + 1u] = vertex_pos.y;
        vertices[vertex_base_index + 2u] = vertex_pos.z;
        
        // STEP 16: Mark this vertex as valid
        // This flag will be used in the compaction step
        vertex_valid[cell_index] = 1u;
    } else {
        // STEP 17: No crossings found - no vertex here
        vertex_valid[cell_index] = 0u;
    }
}

// SUMMARY OF WHAT THIS SHADER DOES:
// 1. Each thread processes one cell in the 3D grid
// 2. For each cell, we check all 12 edges
// 3. If an edge crosses the isosurface (sign change in density), we calculate the crossing point
// 4. We average all crossing points to get a single vertex position
// 5. We store this vertex and mark it as valid
// 6. Cells with no crossings are marked as invalid
//
// OUTPUT:
// - vertices: array of Vec3 positions (some invalid, will be compacted later)
// - vertex_valid: array of flags (1 = has vertex, 0 = no vertex)
