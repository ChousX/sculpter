// ============================================
// KERNEL 5: Compact Faces
// ============================================
// This shader removes invalid faces from the array, creating a dense
// packed array. Similar to compact vertices, but operates on quads (4 indices).

// STEP 1: Define bind group
@group(0) @binding(0)
var<storage, read> faces: array<u32>;  // Input: sparse faces (4 vertex indices per face, with gaps)

@group(0) @binding(1)
var<storage, read> face_valid: array<u32>;  // Input: validity flags (1 = valid face)

@group(0) @binding(2)
var<storage, read> face_indices: array<u32>;  // Input: compacted indices from prefix sum

@group(0) @binding(3)
var<storage, read_write> compacted_faces: array<u32>;  // Output: dense face array

// STEP 2: Define workgroup size
// Using 256 threads for 1D processing of the face array
@compute @workgroup_size(256, 1, 1)
fn compact_faces(
    @builtin(global_invocation_id) global_id: vec3<u32>,
) {
    // STEP 3: Get thread index
    let thread_idx = global_id.x;
    
    // STEP 4: Bounds check
    // Make sure we're within the face validity array bounds
    if (thread_idx >= arrayLength(&face_valid)) {
        return;
    }
    
    // STEP 5: Check if this face is valid
    // Only process faces that were marked as valid in the generation step
    if (face_valid[thread_idx] != 0u) {
        // STEP 6: Get the compacted index
        // The prefix sum told us where this face should go in the output
        let compacted_idx = face_indices[thread_idx];
        
        // STEP 7: Copy face data from sparse to dense array
        // Each face is a quad with 4 vertex indices, so 4 u32 values
        let src_base = thread_idx * 4u;      // Source position in sparse array
        let dst_base = compacted_idx * 4u;   // Destination position in dense array
        
        // Copy all four vertex indices that define this quad face
        compacted_faces[dst_base + 0u] = faces[src_base + 0u];  // Vertex 0 (bottom-left)
        compacted_faces[dst_base + 1u] = faces[src_base + 1u];  // Vertex 1 (bottom-right)
        compacted_faces[dst_base + 2u] = faces[src_base + 2u];  // Vertex 2 (top-right)
        compacted_faces[dst_base + 3u] = faces[src_base + 3u];  // Vertex 3 (top-left)
    }
    // STEP 8: Invalid faces are simply skipped
    // No else clause needed - invalid entries are not copied
}

// ============================================
// EXAMPLE WALKTHROUGH
// ============================================
// Input faces (sparse):
//   Index: 0              1              2              3              4              5
//   Data:  [0,1,2,3]      [0,0,0,0]      [4,5,6,7]      [8,9,10,11]    [0,0,0,0]      [12,13,14,15]
//
// Validity flags:
//   [1, 0, 1, 1, 0, 1]
//
// Prefix sum indices (from previous prefix sum on face_valid):
//   [0, 0, 1, 2, 2, 3]
//
// Compaction process:
//   Thread 0: valid=1, index=0 -> Copy [0,1,2,3] to position 0
//   Thread 1: valid=0 -> Skip
//   Thread 2: valid=1, index=1 -> Copy [4,5,6,7] to position 1
//   Thread 3: valid=1, index=2 -> Copy [8,9,10,11] to position 2
//   Thread 4: valid=0 -> Skip
//   Thread 5: valid=1, index=3 -> Copy [12,13,14,15] to position 3
//
// Output faces (dense):
//   Index: 0              1              2              3
//   Data:  [0,1,2,3]      [4,5,6,7]      [8,9,10,11]    [12,13,14,15]
//
// Result: We've removed all the invalid face slots and created a
// tightly packed array! This is now ready to be used as index buffer
// for rendering the mesh.
//
// ============================================
// CONVERTING QUADS TO TRIANGLES
// ============================================
// Note: This shader outputs quads (4 vertices per face), but most
// rendering systems expect triangles (3 vertices per face).
//
// To convert quads to triangles, each quad [v0,v1,v2,v3] becomes
// two triangles:
//   Triangle 1: [v0, v1, v2]
//   Triangle 2: [v0, v2, v3]
//
// This conversion can be done:
// 1. In a post-processing step on the GPU
// 2. On the CPU after readback
// 3. Or by modifying this shader to output triangles directly
//
// The quad representation is more compact (4 indices vs 6 indices),
// which is why Surface Nets traditionally uses it.
