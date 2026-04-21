## Attentions
Always ask when you are having trouble accessing a tool through prompt or when you want to load a library that is not planend.

## Plan: Leptos Front-End Only PicTagger

### Goal
Build a client-side only web app in Rust + Leptos that manages a tagged image gallery and a linked scatter plot (`frequency` vs `IB`).

### Tech Choices
1. Framework: `leptos` CSR mode (no backend).
2. Build tooling: `trunk` for local dev/build.
3. Persistence: browser `localStorage` (JSON serialized records).
4. Plot rendering: SVG-based custom scatter component in Leptos (no heavy chart dependency in first version).
5. Image storage: browser object URLs or data URLs for uploaded files.

### Data Model
Use one record type per image:
1. `id: String` (UUID)
2. `image_data: String` (data URL) or local object URL
3. `thumbnail_data: Option<String>` (optional optimization)
4. `ib: f64`
5. `source: String`
6. `index: i32`
7. `frequency: f64`
8. `weight: f64`
9. `created_at: i64`
10. `updated_at: i64`

### Core UI Structure
Single page with 3 logical panels:
1. Main : scatter plot panel (fixed height).
2. Bottom center: scrollable gallery list (supports hundreds of items).
3. Bootm right side: fixed details/editor panel for selected image.

### Functional Plan
1. Gallery CRUD:
   - Add images via file picker (multi-select).
   - Create default metadata for each new image.
   - Edit metadata in side panel.
   - Delete selected image with confirmation.
2. Selection behavior:
   - Clicking gallery item selects it.
   - Selecting from scatter point also selects it.
   - Side panel always reflects currently selected image.
3. Filtering:
   - Add IB range filter (`min_ib`, `max_ib`).
   - Gallery list and scatter plot both use filtered set.
4. Scatter plot:
   - X axis = `IB`, Y axis = `frequency`.
   - One point per visible image.
   - Hover point shows small preview + key fields.
   - Hover card actions:
     - `Jump to item` (scroll into gallery item).
     - `Open details` (select item and focus side panel).
5. Dynamic updates:
   - Any metadata edit immediately updates both list and scatter point position.
6. Persistence:
   - Auto-save on every CRUD/edit action.
   - Load from `localStorage` on app startup.
   - Add clear/reset action for local data.
   - Allow package, save and load data on local storage.

### Component Breakdown
1. `AppShell`:
   - Shared signals/state wiring.
2. `GalleryList`:
   - Virtual/efficient rendering strategy if count grows.
3. `ImageCard`:
   - Thumbnail + compact metadata summary.
4. `DetailsPanel`:
   - Editable form for all fields.
5. `IbFilterBar`:
   - IB min/max controls + reset.
6. `ScatterPlot`:
   - Axis scaling, point rendering, hover interactions.
7. `StorageService`:
   - Serialize/deserialize records to `localStorage`.

### State Design (Leptos)
1. `images: RwSignal<Vec<ImageRecord>>`
2. `selected_id: RwSignal<Option<String>>`
3. `filter_ib_min: RwSignal<Option<f64>>`
4. `filter_ib_max: RwSignal<Option<f64>>`
5. Memoized `filtered_images` derived from `images` + filters.
6. CRUD/update helpers centralized to ensure persistence and consistent updates.

### Milestones
1. M1: Project scaffold + data model + localStorage load/save.
2. M2: Gallery list + selection + add/delete.
3. M3: Side panel metadata editing with reactive updates.
4. M4: IB filtering connected to list.
5. M5: Scatter plot rendering and point hover preview.
6. M6: Cross-navigation (plot to list + panel) and polish.
7. M7: Testing, edge-case handling, and UX refinements.

### Validation Checklist
1. Add 300+ images and maintain acceptable scroll/interaction responsiveness.
2. Metadata edits reflect in list and scatter in real time.
3. IB filter affects both gallery and plot consistently.
4. Hovering a point shows correct image preview.
5. Plot actions correctly select and scroll to target image.
6. Page refresh restores gallery from localStorage.
7. Delete and reset flows do not leave stale selection state.

### Risks and Mitigations
1. `localStorage` size limits with many large images:
   - Mitigate by downscaling/compressing images before storing.
2. Performance with hundreds of DOM nodes:
   - Mitigate with lazy image loading and optional list virtualization.
3. Numeric input quality (`f64`, `i32` parsing):
   - Use validated form inputs and inline error states.
