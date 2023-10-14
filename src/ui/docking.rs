use std::os::raw::c_char;

pub struct DockNode {
    id: u32,
}

impl DockNode {
    fn new(id: u32) -> Self {
        Self { id }
    }

    pub fn is_split(&self) -> bool {
        unsafe {
            let node = imgui::sys::igDockBuilderGetNode(self.id);
            if node.is_null() {
                false
            } else {
                imgui::sys::ImGuiDockNode_IsSplitNode(node)
            }
        }
    }
    /// Dock window into this dockspace
    #[doc(alias = "DockBuilder::DockWindow")]
    pub fn dock_window(&self, window: &str) {
        let window = imgui::ImString::from(window.to_string());
        unsafe { imgui::sys::igDockBuilderDockWindow(window.as_ptr(), self.id) }
    }

    #[doc(alias = "DockBuilder::SplitNode")]
    pub fn split<D, O>(&self, split_dir: imgui::Direction, size_ratio: f32, dir: D, opposite_dir: O)
    where
        D: FnOnce(DockNode),
        O: FnOnce(DockNode),
    {
        if self.is_split() {
            // Can't split an already split node (need to split the
            // node within)
            return;
        }

        let mut out_id_at_dir: imgui::sys::ImGuiID = 0;
        let mut out_id_at_opposite_dir: imgui::sys::ImGuiID = 0;
        unsafe {
            imgui::sys::igDockBuilderSplitNode(
                self.id,
                split_dir as i32,
                size_ratio,
                &mut out_id_at_dir,
                &mut out_id_at_opposite_dir,
            );
        }

        dir(DockNode::new(out_id_at_dir));
        opposite_dir(DockNode::new(out_id_at_opposite_dir));
    }
}

/// # Docking

pub struct UiDocking {}

impl UiDocking {
    #[doc(alias = "IsWindowDocked")]
    pub fn is_window_docked(&self) -> bool {
        unsafe { imgui::sys::igIsWindowDocked() }
    }
    /// Create dockspace with given label. Returns a handle to the
    /// dockspace which can be used to, say, programatically split or
    /// dock windows into it
    #[doc(alias = "DockSpace")]
    pub fn dockspace(&self, label: &str) -> DockNode {
        let label = imgui::ImString::from(label.to_string());
        unsafe {
            let id = imgui::sys::igGetID_Str(label.as_ptr() as *const c_char);
            imgui::sys::igDockSpace(
                id,
                [0.0, 0.0].into(),
                0,
                std::ptr::null::<imgui::sys::ImGuiWindowClass>(),
            );
            DockNode { id }
        }
    }

    #[doc(alias = "DockSpaceOverViewport")]
    pub fn dockspace_over_viewport(&self) {
        unsafe {
            imgui::sys::igDockSpaceOverViewport(
                imgui::sys::igGetMainViewport(),
                0,
                std::ptr::null::<imgui::sys::ImGuiWindowClass>(),
            );
        }
    }
}
