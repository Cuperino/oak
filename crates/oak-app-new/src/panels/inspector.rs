// Oak Video Editor - Non-Linear Video Editor
// Copyright (C) 2026 Oak Team
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <http://www.gnu.org/licenses/>.

//! The inspector panel (检查器·效果栈): the selected clip's effect stack
//! and parameters. Stub — effect chains are not part of the engine's C
//! ABI yet; when they land, this panel becomes the drop target for
//! [`crate::panels::effect_library::LibraryEffectDrag`].

crate::panels::stub_panel!(InspectorPanel, "inspector", "panel.inspector");
