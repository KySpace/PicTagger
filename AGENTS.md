## Attentions
Always ask when you are having trouble accessing a tool through prompt or when you want to load a library that is not planned.

This project is for local data processing only. All images must come from local files. Do not add camera capture, webcam capture, or any device-camera workflow.

## Project: PicTagger

PicTagger is a client-side only Rust + Leptos web app for local image tagging and visual inspection. It manages locally loaded images, editable metadata, tag colors, and a linked scatter plot of `frequency` vs `IB`.

## Tech Stack

1. Framework: `leptos` in CSR mode.
2. Build tooling: `trunk`.
3. Persistence: browser `localStorage` for active working cache.
4. Import/export: ZIP cache containing `cache.yaml` plus image files.
5. Plot rendering: Plotly scatter plot via `src/plotly_bridge.js`.
6. Image loading: browser file picker only, using data URLs for display.

## Current Data Model

Each image is represented by `ImageRecord` in `src/models.rs`.

Main fields:

1. `id: Uuid`
2. `image_data: String`
3. `image_path: String`
4. `ib: f64`
5. `source: String`
6. `source_tag: String`
7. `tags: Vec<String>` with up to two tags per image
8. `index: i32`
9. `freq_weight_pairs: Vec<FrequencyWeightPair>`
10. `frequency: f64` and `weight: f64` retained only for legacy compatibility
11. `created_at: i64`
12. `updated_at: i64`

Legacy cache files may still contain a scalar `tag` field. Import should read that as `tags` with length 1, but new app state and exports use the vector field.

Each `FrequencyWeightPair` has:

1. `frequency: Option<f64>`
2. `weight: Option<f64>`

Blank pairs are intentionally inactive. A point appears in the scatter plot only when a pair has a frequency value.

## Current UI

The app is a single-page interface with these main areas:

1. Toolbar:
   - Export ZIP.
   - Import Cache from ZIP or legacy YAML.
   - Add Images from local files.
2. IB filter bar:
   - Filters gallery and plot by `IB` range.
3. Top tab area:
   - Scatter Plot tab.
   - Tag Editor tab.
4. Main content grid:
   - Gallery list.
   - Details metadata editor.
   - On wide screens, a larger detail image panel appears as a third column.

## Implemented Features

### Gallery

1. Add one or more local image files.
2. Batch import modal for multi-image add:
   - Shared `source_tag`.
   - Shared `IB`.
   - Auto-index from filename number when unique in the import batch.
3. Select image from gallery.
4. Delete selected image.
5. Clear Gallery with confirmation, removing pictures and plotted dots while keeping tags.
6. Gallery cards gray out when an image has no active frequencies.
7. Gallery cards show the primary tag color as a small indicator.
8. Gallery cards update when details are edited.

### Details Panel

1. Edit source, source tag, IB, index, and up to two tags.
2. Tag fields use two dropdown selectors on one row.
3. If the first selector is "No tag" and the second selector has a tag, the second tag becomes the primary tag.
4. Multiple frequency/weight pairs per image.
5. Three editable blank frequency/weight rows are shown by default.
6. Add more frequency/weight rows.
7. Clear all frequency/weight rows.
8. Source, source tag, IB, index, and frequency/weight text fields buffer input locally and commit on change/blur or Enter to avoid input lag.
9. Double-click the detail image to open a larger popup preview.
10. Detail image uses full-image display with preserved aspect ratio, not cropping.
11. Details and preview surfaces show colored tag disks, including split disks for two-tag images.

### Scatter Plot

1. Plotly plot of frequency vs IB with zoom, pan, and scroll zoom.
2. Supports multiple plotted points per image, one point per active frequency pair.
3. X and Y axes can be switched between linear and log scales independently.
4. Axis limits can be manually entered through a hidden menu and are persisted to `localStorage`.
5. Changing the tag filter preserves the current Plotly zoom/pan range.
6. Point color comes from the primary tag when no tag filter is active.
7. When filtering by a tag, matching hybrid-tagged dots use the filtered tag color.
8. The plot has a tag filter selector, including "All" and "No tag".
9. Default mode uses weight-proportional opacity.
10. Opaque mode uses a slider-controlled weight threshold:
   - Points at or above threshold are filled and fully opaque.
   - Points below threshold remain visible as thin colored-edge hollow dots.
11. Hovering a point shows a cursor-positioned image preview card.
12. Hovering a point increases that dot radius by 20%.
13. Hover card provides:
   - Jump to gallery item.
   - Open details.
14. Hover card lingers while the mouse remains near or over the card and is guarded against stale Plotly unhover timers.
15. Plotly's text-only hover card is disabled in favor of the custom preview card.

### Tag Editor

1. Manage the list of tags.
2. Edit tag names.
3. Assign colors using OKLCH hue values.
4. Tag colors propagate to gallery cards, details selector, and scatter plot points.
5. New tags can be added up to `MAX_TAGS`.

### Cache Import/Export

1. Active working data is auto-saved to browser `localStorage`.
2. Export creates `pictagger-cache.zip`.
3. ZIP export writes:
   - `cache.yaml`
   - image files grouped into folders named by sanitized `source_tag`
4. YAML image paths point to the image paths inside the ZIP.
5. ZIP import restores image bytes into displayable data URLs.
6. Legacy `.yaml` and `.yml` imports are still accepted, but those files may not restore images unless image data is embedded or paths are displayable.
7. Importing a cache replaces the current gallery and tag definitions with the imported cache contents.
8. Clear Gallery removes only current image records/dots; it does not reset tag definitions.

## Important Files

1. `src/app.rs`
   - Main app state and top-level wiring.
   - File picker, batch import modal, ZIP import/export, delete-all modal.
2. `src/models.rs`
   - `ImageRecord`, `FrequencyWeightPair`, tag definitions, default values.
3. `src/storage.rs`
   - `localStorage` persistence.
   - ZIP cache export/import.
   - Legacy YAML import.
4. `src/components/gallery_list.rs`
   - Gallery list and image cards.
5. `src/components/details_panel.rs`
   - Metadata editor, frequency/weight editor, large image popup.
6. `src/components/scatter_plot.rs`
   - Plotly payload/state, axis controls, tag filter, threshold mode, hover card.
7. `src/components/tag_editor.rs`
   - Tag management UI.
8. `src/components/filter_bar.rs`
   - IB filter controls.
9. `style.css`
   - Global layout and component styling.

## Validation Commands

Use these after code changes:

```powershell
cargo check --target wasm32-unknown-unknown
trunk build
```

For local manual testing:

```powershell
trunk serve --address 127.0.0.1 --port 8080
```

## Recent Session Progress

Completed in the latest session:

1. Optimized Details panel input performance by buffering text inputs and debouncing active cache saves.
2. Replaced the main scatter plot with Plotly for zooming and panning.
3. Added independent X/Y scale selectors.
4. Added tag filtering to the main plot.
5. Added opaque threshold mode with a weight slider and hollow low-weight dots.
6. Fixed tag color propagation so scatter plot colors match tag editor colors.
7. Added support for up to two tags per image using `tags: Vec<String>`.
8. Updated cache import/export compatibility for legacy scalar `tag` fields.
9. Added split colored disks for two-tag images in details and preview cards.
10. Stabilized cursor-positioned hover preview cards and disabled Plotly text hover.
11. Preserved Plotly zoom/pan when changing the plot tag filter.
12. Changed Delete All to Clear Gallery, which clears pictures/dots but keeps tags.

## Details Input Performance Plan

The Details panel can become laggy because several fields still commit to the global `images` signal on every keystroke. Each commit can trigger full gallery serialization to `localStorage`, filtered image recomputation, gallery updates, and scatter plot recomputation.

Progress and remaining optimization order:

1. Completed: buffer Details text inputs locally.
   - Use local `RwSignal<String>` draft values for `source`, `source_tag`, `ib`, and `index`.
   - Update draft values on `input`.
   - Commit to `images` only on `change`/blur or Enter.
   - Keep the existing frequency/weight behavior, which already commits on change/blur to avoid input lag.
2. Completed: debounce active cache persistence.
   - Replace immediate `save_records(&images.get())` on every `images` change with a short debounced save.
   - This avoids synchronous `serde_json::to_string` and `localStorage.set_item` work during typing.
3. Remaining: split metadata persistence from image data persistence.
   - Store frequently edited metadata separately from large image data URLs.
   - Rewrite image data only when images are added, imported, deleted, or reset.
   - Metadata edits should not rewrite hundreds of embedded image strings.
4. Remaining: reduce gallery recomputation.
   - Avoid repeated `images.get().into_iter().find(...)` lookups inside each gallery card.
   - Pass card data directly, use a keyed map, or introduce more granular per-record state.
5. Remaining: consider granular record state.
   - A single `RwSignal<Vec<ImageRecord>>` invalidates gallery, details, filters, and scatter plot together.
   - Per-record signals, a Leptos store, or a metadata map keyed by `Uuid` would let edits to one selected record avoid waking unrelated UI.

## Known Risks

1. Browser `localStorage` can hit size limits when many large images are stored as data URLs.
2. ZIP export is the preferred durable backup path because it stores actual image bytes.
3. Hundreds of images are expected, but gallery virtualization has not been implemented.
4. Numeric fields intentionally avoid committing on every keystroke; this improves typing performance but means values update after change/blur.
