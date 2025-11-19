// ============================================
// KERNEL 2: Prefix Sum (Parallel Scan)
// ============================================
// This shader computes a prefix sum (also called scan) to compact arrays.
// It converts an array of flags [1,0,1,1,0,1] into indices [0,0,1,2,2,3]
// This tells us where each valid element should go in the compacted array.

// STEP 1: Define bind group
@group(0) @binding(0)
var<storage, read> input: array<u32>;  // Input: validity flags (0 or 1)

@group(0) @binding(1)
var<storage, read_write> output: array<u32>;  // Output: compacted indices

@group(0) @binding(2)
var<storage, read_write> total_count: array<u32>;  // Output: total number of valid elements

// STEP 2: Define workgroup parameters
// We use 256 threads per workgroup for 1D processing
const WORKGROUP_SIZE: u32 = 256u;

// STEP 3: Shared memory for parallel reduction
// Each workgroup shares this memory for efficient parallel computation
// This is work-efficient prefix sum using the Blelloch algorithm
var<workgroup> shared_data: array<u32, WORKGROUP_SIZE>;

@compute @workgroup_size(256, 1, 1)
fn prefix_sum(
    @builtin(global_invocation_id) global_id: vec3<u32>,
    @builtin(local_invocation_id) local_id: vec3<u32>,
    @builtin(workgroup_id) workgroup_id: vec3<u32>,
) {
    // STEP 4: Get thread indices
    let global_idx = global_id.x;      // Unique index across all threads
    let local_idx = local_id.x;         // Index within this workgroup (0-255)
    
    // STEP 5: Load input into shared memory
    // Each thread loads one element from global memory to shared memory
    // This is much faster for the subsequent operations
    if (global_idx < arrayLength(&input)) {
        shared_data[local_idx] = input[global_idx];
    } else {
        // Threads beyond the array length use 0
        shared_data[local_idx] = 0u;
    }
    
    // STEP 6: Wait for all threads in workgroup to finish loading
    // workgroupBarrier() ensures all threads reach this point before continuing
    // This is critical for correctness - we need all data loaded before processing
    workgroupBarrier();
    
    // ============================================
    // BLELLOCH SCAN ALGORITHM
    // ============================================
    // This is a work-efficient parallel scan algorithm with two phases:
    // 1. Up-sweep (reduce): Build a tree of partial sums
    // 2. Down-sweep: Traverse the tree to compute prefix sums
    
    // STEP 7: UP-SWEEP PHASE (Build partial sums tree)
    // This builds a binary tree where each parent is the sum of its children
    // Example for 8 elements:
    //   Level 0: [1,0,1,1,0,1,0,1]
    //   Level 1: [1,1,2,1,1,1]      (pairs summed)
    //   Level 2: [2,3,2]            (pairs summed again)
    //   Level 3: [5,2]              (almost done)
    //   Final:   [7]                (total sum)
    
    var offset = 1u;
    // Loop through tree levels (log2(WORKGROUP_SIZE) levels)
    for (var d = WORKGROUP_SIZE >> 1u; d > 0u; d = d >> 1u) {
        // Only some threads work at each level (d threads)
        if (local_idx < d) {
            // Calculate which elements this thread is combining
            let ai = offset * (2u * local_idx + 1u) - 1u;  // Left child index
            let bi = offset * (2u * local_idx + 2u) - 1u;  // Right child index
            
            // Sum: parent = left_child + right_child
            shared_data[bi] = shared_data[bi] + shared_data[ai];
        }
        offset = offset * 2u;
        
        // Wait for this level to complete before starting next level
        workgroupBarrier();
    }
    
    // STEP 8: Clear the last element (root of tree)
    // For exclusive scan, we want the root to be 0
    // This makes the algorithm produce: [0, 1, 1, 2, 3, 3, 4]
    // instead of inclusive scan: [1, 1, 2, 3, 3, 4, 4]
    if (local_idx == 0u) {
        // Store the total count before clearing
        if (workgroup_id.x == 0u) {
            total_count[0] = shared_data[WORKGROUP_SIZE - 1u];
        }
        shared_data[WORKGROUP_SIZE - 1u] = 0u;
    }
    
    workgroupBarrier();
    
    // STEP 9: DOWN-SWEEP PHASE (Compute prefix sums)
    // This traverses back down the tree, computing prefix sums
    // At each level, we:
    //   1. Copy parent value to right child
    //   2. Add old right child value to get new parent (for left child)
    // Example for 8 elements:
    //   Start:   [2,3,2,0]
    //   Level 1: [2,2,2,0]          (sweep down)
    //   Level 2: [2,2,0,2,2,0]      (sweep down more)
    //   Final:   [0,1,1,2,3,3,4,4]  (exclusive prefix sum)
    
    for (var d = 1u; d < WORKGROUP_SIZE; d = d * 2u) {
        offset = offset >> 1u;
        
        if (local_idx < d) {
            let ai = offset * (2u * local_idx + 1u) - 1u;
            let bi = offset * (2u * local_idx + 2u) - 1u;
            
            // Swap and add pattern
            let temp = shared_data[ai];
            shared_data[ai] = shared_data[bi];      // Right child gets parent value
            shared_data[bi] = shared_data[bi] + temp;  // Parent gets old right + parent
        }
        
        workgroupBarrier();
    }
    
    // STEP 10: Write results back to global memory
    // Now shared_data contains the exclusive prefix sum
    if (global_idx < arrayLength(&output)) {
        output[global_idx] = shared_data[local_idx];
    }
}

// ============================================
// EXAMPLE WALKTHROUGH
// ============================================
// Input:  [1, 0, 1, 1, 0, 1, 0, 1]  (validity flags)
// Output: [0, 0, 1, 2, 2, 3, 3, 4]  (compacted indices)
//
// Interpretation:
// - Element 0 (valid=1) goes to index 0 in compacted array
// - Element 1 (valid=0) is skipped
// - Element 2 (valid=1) goes to index 1 in compacted array
// - Element 3 (valid=1) goes to index 2 in compacted array
// - Element 4 (valid=0) is skipped
// - Element 5 (valid=1) goes to index 3 in compacted array
// - Element 6 (valid=0) is skipped
// - Element 7 (valid=1) goes to index 4 in compacted array
//
// Total count: 5 valid elements
//
// ============================================
// LIMITATIONS OF THIS IMPLEMENTATION
// ============================================
// This simple version only works within a single workgroup (256 elements).
// For arrays larger than 256, you need a multi-pass algorithm:
// 1. Compute prefix sum within each workgroup
// 2. Store the sum of each workgroup
// 3. Compute prefix sum of workgroup sums
// 4. Add workgroup offsets to each element
//
// For now, we'll process in chunks and the Rust side will need to handle
// this limitation or we'll need to implement a multi-level scan.
