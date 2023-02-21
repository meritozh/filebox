// Copyright (c) 2023 meritozh
//
// This software is released under the MIT License.
// https://opensource.org/licenses/MIT

pub(crate) struct Executor<F> {
    tasks: Vec<Box<F>>,
}

impl<F> Executor<F>
where
    F: Fn(&[u8]) -> String,
{
    pub(crate) fn new() -> Self {
        Self { tasks: Vec::new() }
    }

    pub(crate) fn add_task(&mut self, task: F) {
        self.tasks.push(Box::new(task));
    }

    pub(crate) fn exec(&self) {
        let mut iter = self.tasks.iter().peekable();

        if iter.peek().is_some() {
            let mut output: String = "test".into();
            output = iter.next().unwrap().call((output.as_bytes(),));
        }
    }
}
