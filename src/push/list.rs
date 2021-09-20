use crate::push::instructions::Instruction;
use crate::push::instructions::InstructionCache;
use crate::push::item::Item;
use crate::push::state::PushState;
use crate::push::topology::Topology;
use std::collections::HashMap;

/// Integer numbers (that is, numbers without decimal points).
pub fn load_list_instructions(map: &mut HashMap<String, Instruction>) {
    map.insert(String::from("LIST.ADD"), Instruction::new(list_add));
    map.insert(String::from("LIST.GET"), Instruction::new(list_get));
    map.insert(String::from("LIST.SET"), Instruction::new(list_set));
    map.insert(
        String::from("LIST.NEIGHBORS"),
        Instruction::new(list_neighbors),
    );
}

pub fn generate_list(push_state: &mut PushState) -> Option<Vec<Item>> {
    if let Some(stack_ids) = push_state.int_vector_stack.pop() {
        let mut items = vec![];
        for &sid in &stack_ids.values {
            match sid {
                BOOL_STACK_ID => {
                    if let Some(bi) = push_state.bool_stack.pop() {
                        items.push(Item::bool(bi));
                    }
                }
                BOOL_VECTOR_STACK_ID => {
                    if let Some(bvi) = push_state.bool_vector_stack.pop() {
                        items.push(Item::boolvec(bvi));
                    }
                }
                CODE_STACK_ID => {
                    if let Some(ci) = push_state.code_stack.pop() {
                        items.push(ci);
                    }
                }
                EXEC_STACK_ID => {
                    if let Some(ei) = push_state.exec_stack.pop() {
                        items.push(ei);
                    }
                }
                FLOAT_STACK_ID => {
                    if let Some(fi) = push_state.float_stack.pop() {
                        items.push(Item::float(fi));
                    }
                }
                FLOAT_VECTOR_STACK_ID => {
                    if let Some(fvi) = push_state.float_vector_stack.pop() {
                        items.push(Item::floatvec(fvi));
                    }
                }
                INT_STACK_ID => {
                    if let Some(ii) = push_state.int_stack.pop() {
                        items.push(Item::int(ii));
                    }
                }
                INT_VECTOR_STACK_ID => {
                    if let Some(ivi) = push_state.int_vector_stack.pop() {
                        items.push(Item::intvec(ivi));
                    }
                }
                NAME_STACK_ID => {
                    if let Some(ni) = push_state.name_stack.pop() {
                        items.push(Item::name(ni));
                    }
                }
                _ => (),
            }
        }
        return Some(items);
    }
    return None;
}

/// LIST.ADD: Pushes a list item to the code stack with the content
/// specified by the top item of the INTVECTOR. Each entry of the INTVECTOR
/// represents the stack id of an item to be contained.
pub fn list_add(push_state: &mut PushState, instruction_set: &InstructionCache) {
    if let Some(items) = generate_list(push_state) {
        let list_item = Item::list(items);
        push_state.code_stack.push(list_item);
    }
}

/// LIST.GET: Pushes a copy of the items bound to the specified index to the execution stack.
/// The index is taken from the top of the INTEGER stack and min-max corrected.
pub fn list_get(push_state: &mut PushState, _instruction_cache: &InstructionCache) {
    if let Some(index) = push_state.int_stack.pop() {
        let size = push_state.code_stack.size() as i32;
        let list_index = i32::max(i32::min(size - 1, index), 0) as usize;
        if let Some(list) = push_state.code_stack.copy(list_index) {
            push_state.exec_stack.push(list);
        }
    }
}

/// LIST.SET: Replaces the items bound to the specified index. The list index is taken
/// from the top of the INTEGER stack and min-max corrected. The content is taken from
/// the stacks that are identified by the top INTVECTOR element. For example
/// [ INTEGER.ID INTEGER.ID BOOLEAN.ID ] = [ 9 9 1 ] replaces with a list containing
/// two integer and a boolean item.
///
pub fn list_set(push_state: &mut PushState, _instruction_cache: &InstructionCache) {
    if let Some(index) = push_state.int_stack.pop() {
        let size = push_state.code_stack.size() as i32;
        let list_index = i32::max(i32::min(size - 1, index), 0) as usize;
        if let Some(items) = generate_list(push_state) {
            let list_item = Item::list(items);
            push_state.code_stack.replace(list_index, list_item);
        }
    }
}

/// LIST.NEIGHBORS: Calculates the neighborhood for a given index element and length. It
/// pushes the indices that are contained in this neighborhood to the INTEGER stack.
/// The size, the number of dimensions and index (vector topology) are taken from the INTEGER
/// stack in that order. The radius is taken from the float stack. Distances are calculated using the
/// Eucledian metric. All values are corrected by max-min. If the size of the top element is not a power
/// of the dimensions the smallest hypercube that includes the indices is used to represent the
/// topology, e.g. two dimensions and size = 38 is represented by[7,7]. Neighbor indices that
/// do no exist (e.g. 40) are ignored.
pub fn list_neighbors(push_state: &mut PushState, _instruction_cache: &InstructionCache) {
    if let Some(topology) = push_state.int_stack.pop_vec(3) {
        let size = i32::max(topology[2], 0);
        let index = i32::max(i32::min(size - 1, topology[1]), 0) as usize;
        let dimensions = i32::max(i32::min(size, topology[0]), 0) as usize;
        if let Some(fval) = push_state.float_stack.pop() {
            let radius = f32::max(fval, 0.0);
            if let Some(neighbors) =
                Topology::find_neighbors(&(size as usize), &dimensions, &index, &radius)
            {
                for n in neighbors.values.iter() {
                    push_state.int_stack.push(*n);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    pub fn icache() -> InstructionCache {
        InstructionCache::new(vec![])
    }
    #[test]
    fn index_vector_neighbors_pushes_result_for_valid_index() {
        let mut test_state = PushState::new();
        test_state.float_stack.push(1.5); // Radius
        test_state.int_stack.push(2); // Dimensions
        test_state.int_stack.push(50); // Index
        test_state.int_stack.push(100); // Size
        list_neighbors(&mut test_state, &icache());
        assert_eq!(
            test_state.int_stack.to_string(),
            String::from("1:61; 2:60; 3:51; 4:50; 5:41; 6:40;")
        );
    }

    #[test]
    fn index_vector_neighbors_corrects_out_of_bounds_index() {
        let mut test_state = PushState::new();
        test_state.float_stack.push(1.5); // Radius
        test_state.int_stack.push(2); // Dimensions
        test_state.int_stack.push(105); // Index
        test_state.int_stack.push(100); // Size
        list_neighbors(&mut test_state, &icache());
        assert_eq!(
            test_state.int_stack.to_string(),
            String::from("1:99; 2:98; 3:89; 4:88;")
        );
        test_state.int_stack.flush();
        test_state.float_stack.push(1.5); // Radius
        test_state.int_stack.push(2); // Dimensions
        test_state.int_stack.push(-10); // Index
        test_state.int_stack.push(100); // Size
        list_neighbors(&mut test_state, &icache());
        assert_eq!(
            test_state.int_stack.to_string(),
            String::from("1:11; 2:10; 3:1; 4:0;")
        );
    }
}