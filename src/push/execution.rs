use crate::push::instructions::Instruction;
use crate::push::instructions::InstructionCache;
use crate::push::item::Item;
use crate::push::state::PushState;
use std::collections::HashMap;

/// Code queued for execution. The EXEC stack maintains the execution state of the Push
/// interpreter. Instructions that specifically manipulate the EXEC stack can be used to implement
/// various kinds of control structures. The CODE stack can also be used in this way, but
/// manipulations to the EXEC stack are "live" in the sense that they are manipulating the actual
/// execution state of the interpreter, not just code that might later be executed.
pub fn load_exec_instructions(map: &mut HashMap<String, Instruction>) {
    map.insert(String::from("EXEC.="), Instruction::new(exec_eq));
    map.insert(String::from("EXEC.DEFINE"), Instruction::new(exec_define));
    map.insert(
        String::from("EXEC.DO*COUNT"),
        Instruction::new(exec_do_count),
    );
    map.insert(
        String::from("EXEC.DO*RANGE"),
        Instruction::new(exec_do_range),
    );
    map.insert(
        String::from("EXEC.DO*TIMES"),
        Instruction::new(exec_do_times),
    );
    map.insert(String::from("EXEC.DUP"), Instruction::new(exec_dup));
    map.insert(String::from("EXEC.FLUSH"), Instruction::new(exec_flush));
    map.insert(String::from("EXEC.IF"), Instruction::new(exec_if));
    map.insert(String::from("EXEC.K"), Instruction::new(exec_k));
    map.insert(String::from("EXEC.POP"), Instruction::new(exec_pop));
    map.insert(String::from("EXEC.ROT"), Instruction::new(exec_rot));
    map.insert(String::from("EXEC.S"), Instruction::new(exec_s));
    map.insert(String::from("EXEC.SHOVE"), Instruction::new(exec_shove));
    map.insert(
        String::from("EXEC.STACKDEPTH"),
        Instruction::new(exec_stack_depth),
    );
    map.insert(String::from("EXEC.SWAP"), Instruction::new(exec_swap));
    map.insert(String::from("EXEC.Y"), Instruction::new(exec_y));
    map.insert(String::from("EXEC.YANK"), Instruction::new(exec_yank));
    map.insert(
        String::from("EXEC.YANKDUP"),
        Instruction::new(exec_yank_dup),
    );
}

/// EXEC.=: Pushes TRUE if the top two items on the EXEC stack are equal, or FALSE otherwise.
pub fn exec_eq(push_state: &mut PushState, _instruction_cache: &InstructionCache) {
    if let Some(pv) = push_state.exec_stack.copy_vec(2) {
        push_state
            .bool_stack
            .push(pv[0].to_string() == pv[1].to_string());
    }
}

/// EXEC.DEFINE: Defines the name on top of the NAME stack as an instruction that will push the top
/// item of the EXEC stack back onto the EXEC stack.
pub fn exec_define(push_state: &mut PushState, _instruction_cache: &InstructionCache) {
    if let Some(name) = push_state.name_stack.pop() {
        if let Some(instruction) = push_state.exec_stack.pop() {
            push_state.name_bindings.insert(name, instruction);
        }
    }
}

/// EXEC.DO*COUNT: An iteration instruction that performs a loop (the body of which is taken from
/// the EXEC stack) the number of times indicated by the INTEGER argument, pushing an index (which
/// runs from zero to one less than the number of iterations) onto the INTEGER stack prior to each
/// execution of the loop body. This is similar to CODE.DO*COUNT except that it takes its code
/// argument from the EXEC stack. This should be implemented as a macro that expands into a call to
/// EXEC.DO*RANGE. EXEC.DO*COUNT takes a single INTEGER argument (the number of times that the loop
/// will be executed) and a single EXEC argument (the body of the loop). If the provided INTEGER
/// argument is negative or zero then this becomes a NOOP. Otherwise it expands into:
/// ( 0 <1 - IntegerArg> EXEC.DO*RANGE <ExecArg> )
pub fn exec_do_count(push_state: &mut PushState, _instruction_cache: &InstructionCache) {
    if let Some(int_arg) = push_state.int_stack.pop() {
        if let Some(exec_code) = push_state.exec_stack.pop() {
            if int_arg < 0 {
                return;
            } else {
                let macro_item = Item::list(vec![
                    exec_code,
                    Item::instruction("EXEC.DO*RANGE".to_string()),
                    Item::int(1 - int_arg),
                    Item::int(0),
                ]);
                push_state.exec_stack.push(macro_item);
            }
        }
    }
}

/// EXEC.DO*RANGE: An iteration instruction that executes the top item on the EXEC stack a number
/// of times that depends on the top two integers, while also pushing the loop counter onto the
/// INTEGER stack for possible access during the execution of the body of the loop. This is similar
/// to CODE.DO*COUNT except that it takes its code argument from the EXEC stack. The top integer is
/// the "destination index" and the second integer is the "current index." First the code and the
/// integer arguments are saved locally and popped. Then the integers are compared. If the integers
/// are equal then the current index is pushed onto the INTEGER stack and the code (which is the
/// "body" of the loop) is pushed onto the EXEC stack for subsequent execution. If the integers are
/// not equal then the current index will still be pushed onto the INTEGER stack but two items will
/// be pushed onto the EXEC stack -- first a recursive call to EXEC.DO*RANGE (with the same code
/// and destination index, but with a current index that has been either incremented or decremented
/// by 1 to be closer to the destination index) and then the body code. Note that the range is
/// inclusive of both endpoints; a call with integer arguments 3 and 5 will cause its body to be
/// executed 3 times, with the loop counter having the values 3, 4, and 5. Note also that one can
/// specify a loop that "counts down" by providing a destination index that is less than the
/// specified current index.
pub fn exec_do_range(push_state: &mut PushState, _instruction_cache: &InstructionCache) {
    if let Some(body) = push_state.exec_stack.pop() {
        if let Some(indices) = push_state.int_stack.pop_vec(2) {
            let destination_idx = indices[1];
            let mut current_idx = indices[0];
            if current_idx == destination_idx {
                push_state.int_stack.push(current_idx);
                push_state.exec_stack.push(body);
            } else {
                push_state.int_stack.push(current_idx);
                if current_idx < destination_idx {
                    current_idx += 1;
                } else {
                    current_idx -= 1;
                }
                let updated_loop = Item::list(vec![
                    body.clone(),
                    Item::instruction("EXEC.DO*RANGE".to_string()),
                    Item::int(destination_idx),
                    Item::int(current_idx),
                ]);
                push_state.exec_stack.push(updated_loop);
                push_state.exec_stack.push(body);
            }
        }
    }
}

/// EXEC.DO*TIMES: Like EXEC.DO*COUNT but does not push the loop counter. This should be
/// implemented as a macro that expands into EXEC.DO*RANGE, similarly to the implementation of
/// EXEC.DO*COUNT, except that a call to INTEGER.POP should be tacked on to the front of the loop
/// body code in the call to EXEC.DO*RANGE. This call to INTEGER.POP will remove the loop counter,
/// which will have been pushed by EXEC.DO*RANGE, prior to the execution of the loop body.
pub fn exec_do_times(push_state: &mut PushState, _instruction_cache: &InstructionCache) {
    if let Some(int_arg) = push_state.int_stack.pop_vec(2) {
        if let Some(exec_code) = push_state.exec_stack.pop() {
            let macro_item = Item::list(vec![
                Item::list(vec![
                    exec_code,
                    Item::instruction("INTEGER.POP".to_string()),
                ]),
                Item::instruction("EXEC.DO*RANGE".to_string()),
                Item::int(int_arg[1]), // destination_idx
                Item::int(int_arg[0]), // current_idx
            ]);
            push_state.exec_stack.push(macro_item);
        }
    }
}

/// EXEC.DUP: Duplicates the top item on the EXEC stack. Does not pop its argument (which, if it
/// did, would negate the effect of the duplication!). This may be thought of as a "DO TWICE"
/// instruction.
pub fn exec_dup(push_state: &mut PushState, _instruction_cache: &InstructionCache) {
    if let Some(instruction) = push_state.exec_stack.copy(0) {
        push_state.exec_stack.push(instruction);
    }
}

/// EXEC.FLUSH: Empties the EXEC stack. This may be thought of as a "HALT" instruction.
pub fn exec_flush(push_state: &mut PushState, _instruction_cache: &InstructionCache) {
    push_state.exec_stack.flush();
}

/// EXEC.IF: If the top item of the BOOLEAN stack is TRUE then this removes the second item on the
/// EXEC stack, leaving the first item to be executed. If it is false then it removes the first
/// item, leaving the second to be executed. This is similar to CODE.IF except that it operates on
/// the EXEC stack. This acts as a NOOP unless there are at least two items on the EXEC stack and
/// one item on the BOOLEAN stack.
pub fn exec_if(push_state: &mut PushState, _instruction_cache: &InstructionCache) {
    if let Some(code) = push_state.exec_stack.pop_vec(2) {
        if let Some(exec_first) = push_state.bool_stack.pop() {
            if exec_first {
                // Push first element for execution
                push_state.exec_stack.push(code[1].clone());
            } else {
                // Push second element for execution
                push_state.exec_stack.push(code[0].clone());
            }
        }
    }
}

/// EXEC.K: The Push implementation of the "K combinator". Removes the second item on the EXEC
/// stack.
pub fn exec_k(push_state: &mut PushState, _instruction_cache: &InstructionCache) {
    if let Some(code) = push_state.exec_stack.pop_vec(2) {
        push_state.exec_stack.push(code[1].clone());
    }
}

/// EXEC.POP: Pops the EXEC stack. This may be thought of as a "DONT" instruction.
pub fn exec_pop(push_state: &mut PushState, _instruction_cache: &InstructionCache) {
    push_state.exec_stack.pop();
}

/// EXEC.ROT: Rotates the top three items on the EXEC stack, pulling the third item out and pushing
/// it on top. This is equivalent to "2 EXEC.YANK".
pub fn exec_rot(push_state: &mut PushState, _instruction_cache: &InstructionCache) {
    push_state.exec_stack.yank(2);
}

/// EXEC.S: The Push implementation of the "S combinator". Pops 3 items from the EXEC stack, which
/// we will call A, B, and C (with A being the first one popped). Then pushes a list containing B
/// and C back onto the EXEC stack, followed by another instance of C, followed by another instance
/// of A.
pub fn exec_s(push_state: &mut PushState, _instruction_cache: &InstructionCache) {
    if let Some(code) = push_state.exec_stack.pop_vec(3) {
        let a = &code[2];
        let b = &code[1];
        let c = &code[0];
        let bc = Item::list(vec![c.clone(), b.clone()]);
        push_state.exec_stack.push(bc);
        push_state.exec_stack.push(c.clone());
        push_state.exec_stack.push(a.clone());
    }
}

/// EXEC.SHOVE: Inserts the top EXEC item "deep" in the stack, at the position indexed by the top
/// INTEGER. This may be thought of as a "DO LATER" instruction.
pub fn exec_shove(push_state: &mut PushState, _instruction_cache: &InstructionCache) {
    if let Some(new_pos) = push_state.int_stack.pop() {
        push_state.exec_stack.shove(new_pos as usize);
    }
}

/// EXEC.STACKDEPTH: Pushes the stack depth onto the INTEGER stack.
pub fn exec_stack_depth(push_state: &mut PushState, _instruction_cache: &InstructionCache) {
    push_state
        .int_stack
        .push(push_state.exec_stack.size() as i32);
}

/// EXEC.SWAP: Swaps the top two items on the EXEC stack.
pub fn exec_swap(push_state: &mut PushState, _instruction_cache: &InstructionCache) {
    push_state.exec_stack.shove(1);
}

/// EXEC.Y: The Push implementation of the "Y combinator". Inserts beneath the top item of the EXEC
/// stack a new item of the form "( EXEC.Y <TopItem> )".
pub fn exec_y(push_state: &mut PushState, _instruction_cache: &InstructionCache) {
    if let Some(top_item) = push_state.exec_stack.copy(0) {
        push_state.exec_stack.push(Item::list(vec![
            top_item,
            Item::instruction("EXEC.Y".to_string()),
        ]));
        push_state.exec_stack.shove(1);
    }
}

/// EXEC.YANK: Removes an indexed item from "deep" in the stack and pushes it on top of the stack.
/// The index is taken from the INTEGER stack. This may be thought of as a "DO SOONER" instruction.
pub fn exec_yank(push_state: &mut PushState, _instruction_cache: &InstructionCache) {
    if let Some(idx) = push_state.int_stack.pop() {
        push_state.exec_stack.yank(idx as usize);
    }
}

/// EXEC.YANKDUP: Pushes a copy of an indexed item "deep" in the stack onto the top of the stack,
/// without removing the deep item. The index is taken from the INTEGER stack.
pub fn exec_yank_dup(push_state: &mut PushState, _instruction_cache: &InstructionCache) {
    if let Some(idx) = push_state.int_stack.pop() {
        if let Some(deep_item) = push_state.exec_stack.copy(idx as usize) {
            push_state.exec_stack.push(deep_item);
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
    fn exec_eq_pushes_true_when_elements_equal() {
        let mut test_state = PushState::new();
        test_state.exec_stack.push(Item::int(1));
        test_state.exec_stack.push(Item::int(1));
        exec_eq(&mut test_state, &icache());
        assert_eq!(test_state.exec_stack.size(), 2);
        assert_eq!(test_state.bool_stack.to_string(), "1:true;");
    }

    #[test]
    fn exec_eq_pushes_false_when_elements_unequal() {
        let mut test_state = PushState::new();
        test_state.exec_stack.push(Item::int(1));
        test_state.exec_stack.push(Item::int(2));
        exec_eq(&mut test_state, &icache());
        assert_eq!(test_state.exec_stack.size(), 2);
        assert_eq!(test_state.bool_stack.to_string(), "1:false;");
    }

    #[test]
    fn exec_define_creates_name_binding() {
        let mut test_state = PushState::new();
        test_state.exec_stack.push(Item::int(2));
        test_state.name_stack.push(String::from("TEST"));
        exec_define(&mut test_state, &icache());
        assert_eq!(
            *test_state.name_bindings.get("TEST").unwrap().to_string(),
            Item::int(2).to_string()
        );
    }

    #[test]
    fn exec_do_count_unfolds_to_macro() {
        let mut test_state = PushState::new();
        test_state.exec_stack.push(Item::noop());
        test_state.int_stack.push(3);
        exec_do_count(&mut test_state, &icache());
        assert_eq!(
            test_state.exec_stack.to_string(),
            "1:List: 1:Literal(0); 2:Literal(-2); 3:InstructionMeta(EXEC.DO*RANGE); 4:InstructionMeta(NOOP);;"
        );
    }

    #[test]
    fn exec_do_range_counts_upwards() {
        let mut test_state = PushState::new();
        test_state.exec_stack.push(Item::noop());
        test_state.int_stack.push(3); // Current index
        test_state.int_stack.push(5); // Destination index
        exec_do_range(&mut test_state, &icache());
        assert_eq!(
            test_state.exec_stack.to_string(),
            "1:InstructionMeta(NOOP); 2:List: 1:Literal(4); 2:Literal(5); 3:InstructionMeta(EXEC.DO*RANGE); 4:InstructionMeta(NOOP);;"
        );
        assert_eq!(test_state.int_stack.to_string(), "1:3;");
    }

    #[test]
    fn exec_do_range_counts_downwards() {
        let mut test_state = PushState::new();
        test_state.exec_stack.push(Item::noop());
        test_state.int_stack.push(6); // Current index
        test_state.int_stack.push(1); // Destination index
        exec_do_range(&mut test_state, &icache());
        assert_eq!(
            test_state.exec_stack.to_string(),
            "1:InstructionMeta(NOOP); 2:List: 1:Literal(5); 2:Literal(1); 3:InstructionMeta(EXEC.DO*RANGE); 4:InstructionMeta(NOOP);;"
        );
        assert_eq!(test_state.int_stack.to_string(), "1:6;");
    }

    #[test]
    fn exec_do_times_pops_loop_counter() {
        let mut test_state = PushState::new();
        test_state.exec_stack.push(Item::noop());
        test_state.int_stack.push(6); // Current index
        test_state.int_stack.push(1); // Destination index
        exec_do_times(&mut test_state, &icache());
        assert_eq!(
            test_state.exec_stack.to_string(),
            "1:List: 1:Literal(6); 2:Literal(1); 3:InstructionMeta(EXEC.DO*RANGE); 4:List: 1:InstructionMeta(INTEGER.POP); 2:InstructionMeta(NOOP);;;"
        );
        assert_eq!(test_state.int_stack.to_string(), "");
    }

    #[test]
    fn exec_dup_duplicates_top_element() {
        let mut test_state = PushState::new();
        test_state.exec_stack.push(Item::noop());
        exec_dup(&mut test_state, &icache());
        assert_eq!(
            test_state.exec_stack.to_string(),
            "1:InstructionMeta(NOOP); 2:InstructionMeta(NOOP);"
        );
    }

    #[test]
    fn exec_flush_empties_stack() {
        let mut test_state = PushState::new();
        // Test element is (1 2)'
        test_state
            .exec_stack
            .push(Item::list(vec![Item::int(0), Item::int(2)]));
        test_state
            .exec_stack
            .push(Item::list(vec![Item::int(1), Item::int(2)]));
        exec_flush(&mut test_state, &icache());
        assert_eq!(test_state.int_stack.to_string(), "");
    }

    #[test]
    fn exec_if_pushes_first_item_when_true() {
        let mut test_state = PushState::new();
        test_state.bool_stack.push(true);
        test_state.exec_stack.push(Item::int(2));
        test_state.exec_stack.push(Item::int(1));
        exec_if(&mut test_state, &icache());
        assert_eq!(test_state.exec_stack.to_string(), "1:Literal(1);");
        assert_eq!(test_state.bool_stack.to_string(), "");
    }

    #[test]
    fn exec_if_pushes_second_item_when_false() {
        let mut test_state = PushState::new();
        test_state.bool_stack.push(false);
        test_state.exec_stack.push(Item::int(2));
        test_state.exec_stack.push(Item::int(1));
        exec_if(&mut test_state, &icache());
        assert_eq!(test_state.exec_stack.to_string(), "1:Literal(2);");
        assert_eq!(test_state.bool_stack.to_string(), "");
    }

    #[test]
    fn exec_k_removes_second_item() {
        let mut test_state = PushState::new();
        test_state.bool_stack.push(false);
        test_state.exec_stack.push(Item::int(2));
        test_state.exec_stack.push(Item::int(1));
        exec_k(&mut test_state, &icache());
        assert_eq!(test_state.exec_stack.to_string(), "1:Literal(1);");
    }

    #[test]
    fn exec_pop_removes_first_item() {
        let mut test_state = PushState::new();
        test_state.bool_stack.push(false);
        test_state.exec_stack.push(Item::int(2));
        test_state.exec_stack.push(Item::int(1));
        exec_pop(&mut test_state, &icache());
        assert_eq!(test_state.exec_stack.to_string(), "1:Literal(2);");
    }

    #[test]
    fn exec_rot_shuffles_elements() {
        let mut test_state = PushState::new();
        test_state.exec_stack.push(Item::int(3));
        test_state.exec_stack.push(Item::int(2));
        test_state.exec_stack.push(Item::int(1));
        assert_eq!(
            test_state.exec_stack.to_string(),
            "1:Literal(1); 2:Literal(2); 3:Literal(3);"
        );
        exec_rot(&mut test_state, &icache());
        assert_eq!(
            test_state.exec_stack.to_string(),
            "1:Literal(3); 2:Literal(1); 3:Literal(2);"
        );
    }

    #[test]
    fn exec_s_pushes_elements_in_right_order() {
        let mut test_state = PushState::new();
        test_state.exec_stack.push(Item::int(3));
        test_state.exec_stack.push(Item::int(2));
        test_state.exec_stack.push(Item::int(1));
        assert_eq!(
            test_state.exec_stack.to_string(),
            "1:Literal(1); 2:Literal(2); 3:Literal(3);"
        );
        exec_s(&mut test_state, &icache());
        assert_eq!(
            test_state.exec_stack.to_string(),
            "1:Literal(1); 2:Literal(3); 3:List: 1:Literal(2); 2:Literal(3);;"
        );
    }

    #[test]
    fn exec_shove_inserts_at_right_position() {
        let mut test_state = PushState::new();
        test_state.exec_stack.push(Item::int(4));
        test_state.exec_stack.push(Item::int(3));
        test_state.exec_stack.push(Item::int(2));
        test_state.exec_stack.push(Item::int(1));
        assert_eq!(
            test_state.exec_stack.to_string(),
            "1:Literal(1); 2:Literal(2); 3:Literal(3); 4:Literal(4);"
        );
        test_state.int_stack.push(2);
        exec_shove(&mut test_state, &icache());
        assert_eq!(
            test_state.exec_stack.to_string(),
            "1:Literal(2); 2:Literal(3); 3:Literal(1); 4:Literal(4);"
        );
    }

    #[test]
    fn exec_stack_depth_pushes_size() {
        let mut test_state = PushState::new();
        // Test element is (1 2)'
        test_state
            .exec_stack
            .push(Item::list(vec![Item::int(0), Item::int(2)]));
        test_state
            .exec_stack
            .push(Item::list(vec![Item::int(1), Item::int(2)]));
        exec_stack_depth(&mut test_state, &icache());
        assert_eq!(test_state.int_stack.to_string(), "1:2;");
    }

    #[test]
    fn exec_swaps_top_elements() {
        let mut test_state = PushState::new();
        test_state.exec_stack.push(Item::int(0));
        test_state.exec_stack.push(Item::int(1));
        exec_swap(&mut test_state, &icache());
        assert_eq!(
            test_state.exec_stack.to_string(),
            "1:Literal(0); 2:Literal(1);"
        );
    }

    #[test]
    fn exec_y_inserts_y_copy_beneath_top_element() {
        let mut test_state = PushState::new();
        test_state.exec_stack.push(Item::int(0));
        exec_y(&mut test_state, &icache());
        assert_eq!(
            test_state.exec_stack.to_string(),
            "1:Literal(0); 2:List: 1:InstructionMeta(EXEC.Y); 2:Literal(0);;"
        );
    }

    #[test]
    fn exec_yank_brings_item_to_top() {
        let mut test_state = PushState::new();
        test_state.exec_stack.push(Item::int(5));
        test_state.exec_stack.push(Item::int(4));
        test_state.exec_stack.push(Item::int(3));
        test_state.exec_stack.push(Item::int(2));
        test_state.exec_stack.push(Item::int(1));
        assert_eq!(
            test_state.exec_stack.to_string(),
            "1:Literal(1); 2:Literal(2); 3:Literal(3); 4:Literal(4); 5:Literal(5);"
        );
        test_state.int_stack.push(3);
        exec_yank(&mut test_state, &icache());
        assert_eq!(
            test_state.exec_stack.to_string(),
            "1:Literal(4); 2:Literal(1); 3:Literal(2); 4:Literal(3); 5:Literal(5);"
        );
    }

    #[test]
    fn exec_yank_dup_copies_item_to_top() {
        let mut test_state = PushState::new();
        test_state.exec_stack.push(Item::int(5));
        test_state.exec_stack.push(Item::int(4));
        test_state.exec_stack.push(Item::int(3));
        test_state.exec_stack.push(Item::int(2));
        test_state.exec_stack.push(Item::int(1));
        assert_eq!(
            test_state.exec_stack.to_string(),
            "1:Literal(1); 2:Literal(2); 3:Literal(3); 4:Literal(4); 5:Literal(5);"
        );
        test_state.int_stack.push(3);
        exec_yank_dup(&mut test_state, &icache());
        assert_eq!(
            test_state.exec_stack.to_string(),
            "1:Literal(4); 2:Literal(1); 3:Literal(2); 4:Literal(3); 5:Literal(4); 6:Literal(5);"
        );
    }
}
