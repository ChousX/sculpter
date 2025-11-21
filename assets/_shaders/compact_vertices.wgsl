// ============================================
// KERNEL 3: Compact Vertices
// ============================================
// This shader removes invalid vertices from the array, creating a dense
// packed array with no gaps. It uses the prefix sum indices computed earlier.

// STEP 1: Define bind group
@group(0) @binding(0)
var<storage, read> vertices: array<f32>;  // Input: sparse vertices (x,y,z packed, with gaps)

@group(0) @binding(1)
var<storage, read> vertex_valid: array<u32>;  // Input: validity flags (1 = valid)

@group(0) @binding(2)
var<storage, read> vertex_indices: array<u32>;  // Input: compacted indices from prefix sum

@group(0) @binding(3)
var<storage, read_write> compacted_vertices: array<f32>;  // Output: dense vertex array

// STEP 2: Define workgroup size
// Using 256 threads for 1D processing of the vertex array
@compute @workgroup_size(256, 1, 1)
fn compact_vertices(
    @builtin(global_invocation_id) global_id: vec3<u32>,
) {
    // STEP 3: Get thread index
    let thread_idx = global_id.x;
    
    // STEP 4: Bounds check
    // Make sure we're within the array bounds
    if (thread_idx >= arrayLength(&vertex_valid)) {
        return;
    }
    
    // STEP 5: Check if this vertex is valid
    // Only process vertices that were marked as valid in the generation step
    if (vertex_valid[thread_idx] != 0u) {
        // STEP 6: Get the compacted index
        // The prefix sum told us where this vertex should go in the output
        let compacted_idx = vertex_indices[thread_idx];
        
        // STEP 7: Copy vertex data from sparse to dense array
        // Vertices are stored as [x,y,z,x,y,z,...] so each vertex takes 3 floats
        let src_base = thread_idx * 3u;      // Source position in sparse array
        let dst_base = compacted_idx * 3u;   // Destination position in dense array
        
        // Copy all three components (x, y, z)
        compacted_vertices[dst_base + 0u] = vertices[src_base + 0u];  // x
        compacted_vertices[dst_base + 1u] = vertices[src_base + 1u];  // y
        compacted_vertices[dst_base + 2u] = vertices[src_base + 2u];  // z
    }
    // STEP 8: Invalid vertices are simply skipped
    // No else clause needed - invalid entries are not copied
}

// ============================================
// EXAMPLE WALKTHROUGH
// ============================================
// Input vertices (sparse):
//   Index: 0         1         2         3         4         5
//   Data:  [1,2,3]   [0,0,0]   [4,5,6]   [7,8,9]   [0,0,0]   [10,11,12]
//
// Validity flags:
//   [1, 0, 1, 1, 0, 1]
//
// Prefix sum indices:
//   [0, 0, 1, 2, 2, 3]
//
// Compaction process:
//   Thread 0: valid=1, index=0 -> Copy [1,2,3] to position 0
//   Thread 1: valid=0 -> Skip
//   Thread 2: valid=1, index=1 -> Copy [4,5,6] to position 1
//   Thread 3: valid=1, index=2 -> Copy [7,8,9] to position 2
//   Thread 4: valid=0 -> Skip
//   Thread 5: valid=1, index=3 -> Copy [10,11,12] to position 3
//
// Output vertices (dense):
//   Index: 0         1         2         3
//   Data:  [1,2,3]   [4,5,6]   [7,8,9]   [10,11,12]
//
// Result: We've removed all the invalid vertices and created a
// tightly packed array with no gaps!
