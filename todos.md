# ToDos

## Performance

1. Initial loading time is too slow due to Iced's WGPU startup times
   - Maybe we can resolve this with IPC? Let a main thread hold onto the WGPU instances and just do window show/hides rather than spawning it from scratch every time? Might allow us to preserve the WGPU context between executions.
2. Currently reading everything in as JSON, not the most performant, especially if JSON starts getting bigger. Maybe we can compile the JSON to a binary format for faster loading?

## Functionality

1. Clicking an emoji should add it to your clipboard
2. Search functionality for emojis
3. Category filtering

## Look and Feel

1. Less ugly in general
2. Flexible layout scaling
3. Theming support
