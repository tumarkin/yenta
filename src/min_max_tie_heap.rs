use getset::Getters;
use min_max_heap::MinMaxHeap;

#[derive(Getters)]
pub struct MinMaxTieHeap<T> {
    #[getset(get = "pub")]
    min_max_heap: MinMaxHeap<T>,
    size: usize,
    #[getset(get = "pub")]
    ties: MinMaxHeap<T>,
    are_tied: Box<dyn Fn(&T, &T) -> bool>,
}

impl<T: Ord> MinMaxTieHeap<T> {
    pub fn new(size: usize, are_tied: Box<dyn Fn(&T, &T) -> bool>) -> MinMaxTieHeap<T> {
        MinMaxTieHeap {
            min_max_heap: MinMaxHeap::with_capacity(size),
            ties: MinMaxHeap::new(),
            size,
            are_tied,
        }
    }

    pub fn push(&mut self, element: T) {
        let mmh = &mut self.min_max_heap;

        // Add elements whenever the heap is underutilized
        if mmh.len() < self.size {
            mmh.push(element);
        } else {
            // The minimum will exist because the heap has elements
            let min_element = mmh.peek_min().unwrap();

            // The element belongs in the heap
            if element > *min_element {
                mmh.push(element);
                let min_element = mmh.pop_min().unwrap();
                self.ties.push(min_element);
                self.clean_up_ties();
            }
            // The element belongs in the tie heap
            else if (self.are_tied)(&min_element, &element) {
                self.ties.push(element);
                self.clean_up_ties();
            }
        }
    }

    pub fn into_vec_desc(self) -> Vec<T> {
        let mut v = self.min_max_heap.into_vec_desc();
        v.append(&mut self.ties.into_vec_desc());
        v
    }

    pub fn merge(self, other: Self) -> Self {
        let mut mmth = MinMaxTieHeap::new(self.size, self.are_tied);
        for element in self.min_max_heap {
            mmth.push(element)
        }
        for element in self.ties {
            mmth.push(element)
        }
        for element in other.min_max_heap {
            mmth.push(element)
        }
        for element in other.ties {
            mmth.push(element)
        }
        mmth
    }

    fn clean_up_ties(&mut self) {
        let min_element_in_mmh = &self.min_max_heap.peek_min().unwrap();
        let are_tied = &self.are_tied;

        while !self.ties.is_empty() {
            let min_element_in_ties = self.ties.peek_min().unwrap();
            if !are_tied(&min_element_in_mmh, min_element_in_ties) {
                self.ties.pop_min();
            } else {
                break;
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
    struct Wrapped(i64);

    #[test]
    fn test_0() {
        let wrapped_are_tied: Box<Fn(&Wrapped, &Wrapped) -> bool> =
            Box::new(|a: &Wrapped, b: &Wrapped| (a.0 - b.0).abs() <= 1);
        let mut mmth: MinMaxTieHeap<Wrapped> = MinMaxTieHeap::new(2, wrapped_are_tied);

        for i in vec![1, 2, 2, 2, 3, 3, 3, 4, 5] {
            mmth.push(Wrapped(i));
        }

        let mmh: MinMaxHeap<Wrapped> = MinMaxHeap::new();
        let in_min_max_heap = mmth.min_max_heap.into_vec_desc();
        let in_ties = mmth.ties.into_vec_desc();

        println!("Heap: {:?}", in_min_max_heap);
        println!("Ties: {:?}", in_ties);
        assert_eq!(in_min_max_heap, vec![Wrapped(5), Wrapped(4)]);
        assert_eq!(in_ties, vec![Wrapped(3), Wrapped(3), Wrapped(3)]);
    }
}
