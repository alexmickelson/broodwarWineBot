# RustBot React Web Interface

This React application provides a real-time status dashboard for the RustBot StarCraft bot. It displays worker assignments, military units, larvae assignments, unit orders, build order progress, and map visualization.

## Technology Stack
- **React 19** with TypeScript
- **TanStack Query v5** for API data fetching and caching
- **Tailwind CSS** with custom purple theme matching the original design
- **Vite** for development and building

## Getting Started

### Install Dependencies
```bash
pnpm install
```

### Run Development Server
```bash
pnpm dev
```

### Build for Production
```bash
pnpm build
```

### Preview Production Build
```bash
pnpm preview
```

## Project Structure
```
src/
├── api/
│   ├── service.ts       # API service functions
│   └── hooks.ts         # TanStack Query hooks
├── components/
│   ├── ui/
│   │   └── Common.tsx   # Reusable UI components
│   ├── GameSpeed.tsx
│   ├── WorkerAssignments.tsx
│   ├── MilitaryAssignments.tsx
│   ├── LarvaeAssignments.tsx
│   ├── UnitOrders.tsx
│   ├── BuildOrder.tsx
│   └── MapVisualization.tsx
├── App.tsx
├── main.tsx
└── index.css
```

## API Endpoints
All endpoints are relative to the server running on `window.location.host`:

- `GET /worker-status` - Worker assignments (gathering, scouting, building)
- `GET /military-assignments` - Military unit assignments and orders
- `GET /larvae` - Larvae assignments to build order indices
- `GET /unit-orders` - All unit orders with positions and targets
- `GET /build-order` - Build order list and current progress
- `GET /map` - Map data with walkability, explored areas, units, and resources
- `GET /game-speed` - Current game speed
- `POST /command` - Send commands (e.g., set game speed)

## Key Features

### Real-time Polling
- Configurable poll intervals (100ms, 250ms, 500ms, 1s, 2s)
- Uses TanStack Query's `refetchInterval` for automatic updates
- All data refreshes automatically at the selected interval

### Game Speed Control
- Set game speed: -1 (Default), 0 (Unlimited), 1 (Fast), 42 (Standard)
- Sends POST requests to `/command` endpoint

### Expandable Sections
- All data sections are collapsible
- Worker Assignments expanded by default
- All others collapsed by default

### Custom Tailwind Theme
Matches the original purple theme:
- Primary background: `#1a0d28`
- Secondary background: `#2d1b4e`
- Border colors: `#6b21a8`, `#4c1d95`
- Text colors: `#d4b8f7`, `#c084fc`, `#a855f7`
- Status colors: Success `#22c55e`, Error `#ef4444`
- Assignment colors: Gathering (green), Scouting (blue), Building (orange)

## Expanding the ESLint configuration

If you are developing a production application, we recommend updating the configuration to enable type-aware lint rules:

```js
export default defineConfig([
  globalIgnores(['dist']),
  {
    files: ['**/*.{ts,tsx}'],
    extends: [
      // Other configs...

      // Remove tseslint.configs.recommended and replace with this
      tseslint.configs.recommendedTypeChecked,
      // Alternatively, use this for stricter rules
      tseslint.configs.strictTypeChecked,
      // Optionally, add this for stylistic rules
      tseslint.configs.stylisticTypeChecked,

      // Other configs...
    ],
    languageOptions: {
      parserOptions: {
        project: ['./tsconfig.node.json', './tsconfig.app.json'],
        tsconfigRootDir: import.meta.dirname,
      },
      // other options...
    },
  },
])
```

You can also install [eslint-plugin-react-x](https://github.com/Rel1cx/eslint-react/tree/main/packages/plugins/eslint-plugin-react-x) and [eslint-plugin-react-dom](https://github.com/Rel1cx/eslint-react/tree/main/packages/plugins/eslint-plugin-react-dom) for React-specific lint rules:

```js
// eslint.config.js
import reactX from 'eslint-plugin-react-x'
import reactDom from 'eslint-plugin-react-dom'

export default defineConfig([
  globalIgnores(['dist']),
  {
    files: ['**/*.{ts,tsx}'],
    extends: [
      // Other configs...
      // Enable lint rules for React
      reactX.configs['recommended-typescript'],
      // Enable lint rules for React DOM
      reactDom.configs.recommended,
    ],
    languageOptions: {
      parserOptions: {
        project: ['./tsconfig.node.json', './tsconfig.app.json'],
        tsconfigRootDir: import.meta.dirname,
      },
      // other options...
    },
  },
])
```
