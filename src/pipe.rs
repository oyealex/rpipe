pub(crate) struct Pipe {
    pub(crate) iter: Box<dyn Iterator<Item = String>>,
    // TODO 2026-01-10 01:27 增加特征描述和后续操作的优化
}

impl Iterator for Pipe {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
}

impl Pipe {
    pub(crate) fn op_map(self, f: impl FnMut(String) -> String + 'static) -> Pipe {
        Pipe { iter: Box::new(self.map(f)) }
    }

    pub(crate) fn op_filter(self, f: impl FnMut(&String) -> bool + 'static) -> Pipe {
        Pipe { iter: Box::new(self.filter(f)) }
    }

    pub(crate) fn op_inspect(self, f: impl FnMut(&String) + 'static) -> Pipe {
        Pipe { iter: Box::new(self.inspect(f)) }
    }
}
