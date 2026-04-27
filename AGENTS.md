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
5. Plot rendering: custom SVG scatter plot in Leptos.
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
7. `tag: String`
8. `index: i32`
9. `freq_weight_pairs: Vec<FrequencyWeightPair>`
10. `frequency: f64` and `weight: f64` retained only for legacy compatibility
11. `created_at: i64`
12. `updated_at: i64`

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
5. Delete all images with confirmation.
6. Gallery cards gray out when an image has no active frequencies.
7. Gallery cards show tag color as a small indicator.
8. Gallery cards update when details are edited.

### Details Panel

1. Edit source, source tag, IB, index, and tag.
2. Tag field uses a dropdown-like selector listing all available tags.
3. Tag options show color as a small circle, not as full-row background.
4. Multiple frequency/weight pairs per image.
5. Three editable blank frequency/weight rows are shown by default.
6. Add more frequency/weight rows.
7. Clear all frequency/weight rows.
8. Frequency/weight text fields update model on change/blur rather than on every key input to avoid input lag.
9. Double-click the detail image to open a larger popup preview.
10. Detail image uses full-image display with preserved aspect ratio, not cropping.

### Scatter Plot

1. SVG plot of frequency vs IB.
2. Supports multiple plotted points per image, one point per active frequency pair.
3. Point color comes from the image tag.
4. Point opacity is currently proportional to weight.
5. Hovering a point shows a preview card.
6. Hover card provides:
   - Jump to gallery item.
   - Open details.
7. Hover card lingers while the mouse remains near or over the card.
8. Axis tick labels are shown.
9. Axis limits can be manually entered through a hidden menu.
10. Axis limits are persisted to `localStorage`.

### Tag Editor

1. Manage the list of tags.
2. Edit tag names.
3. Assign colors using OKLCH hue values.
4. Tag colors propagate to gallery cards, details selector, and scatter plot points.

### Cache Import/Export

1. Active working data is auto-saved to browser `localStorage`.
2. Export creates `pictagger-cache.zip`.
3. ZIP export writes:
   - `cache.yaml`
   - image files grouped into folders named by sanitized `source_tag`
4. YAML image paths point to the image paths inside the ZIP.
5. ZIP import restores image bytes into displayable data URLs.
6. Legacy `.yaml` and `.yml` imports are still accepted, but those files may not restore images unless image data is embedded or paths are displayable.

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
   - SVG plot, axis limits, ticks, point rendering, hover card.
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

## Pending Work

The latest interrupted request was not implemented yet:

1. Add scatter plot controls to enable or disable weight-based opacity.
2. When opacity is enabled:
   - All active points are shown.
   - Point opacity remains proportional to weight.
3. When opacity is disabled:
   - Add a slider-controlled weight threshold.
   - Hide points with weight below threshold.
   - Show points at or above threshold at full opacity.

## Known Risks

1. Browser `localStorage` can hit size limits when many large images are stored as data URLs.
2. ZIP export is the preferred durable backup path because it stores actual image bytes.
3. Hundreds of images are expected, but gallery virtualization has not been implemented.
4. Numeric fields intentionally avoid committing on every keystroke; this improves typing performance but means values update after change/blur.
