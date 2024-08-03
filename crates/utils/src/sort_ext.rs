/// Extension trait for sorting vectors and slices by index in-place
pub trait SortExt<T> {
    fn sort_by_index(&mut self, indices: &mut [usize]);
}

impl<T: Clone> SortExt<T> for [T] {
    fn sort_by_index(&mut self, indices: &mut [usize]) {
        for idx in 0..self.len() {
            if indices[idx] != usize::MAX {
                let mut current_idx = idx;
                loop {
                    let target_idx = indices[current_idx];
                    indices[current_idx] = usize::MAX;
                    if indices[target_idx] == usize::MAX {
                        break;
                    }
                    self.swap(current_idx, target_idx);
                    current_idx = target_idx;
                }
            }
        }
    }
}
