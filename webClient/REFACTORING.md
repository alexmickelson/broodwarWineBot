# React Web Refactoring Complete

The codebase has been refactored with the following structure:

## New Structure

Each component now has its own folder with:
- **Component file** (e.g., `BuildOrder.tsx`)
- **hooks.ts** - React Query hooks specific to this component
- **service.ts** - API calls and types specific to this component  
- **index.ts** - Clean exports for the component

## Components

```
src/
├── components/
│   ├── BuildOrder/
│   │   ├── BuildOrder.tsx
│   │   ├── hooks.ts (useBuildOrder)
│   │   ├── service.ts (BuildOrderSnapshot, fetchBuildOrder)
│   │   └── index.ts
│   ├── GameSpeed/
│   │   ├── GameSpeed.tsx
│   │   ├── hooks.ts (useGameSpeed, useSetGameSpeed)
│   │   ├── service.ts (GameSpeedSnapshot, setGameSpeed, fetchGameSpeed)
│   │   └── index.ts
│   ├── LarvaeAssignments/
│   │   ├── LarvaeAssignments.tsx
│   │   ├── hooks.ts (useLarvae)
│   │   ├── service.ts (LarvaeSnapshot, fetchLarvae)
│   │   └── index.ts
│   ├── MapVisualization/
│   │   ├── MapVisualization.tsx
│   │   ├── hooks.ts (useMap)
│   │   ├── service.ts (MapSnapshot, MapData, fetchMap)
│   │   └── index.ts
│   ├── MilitaryAssignments/
│   │   ├── MilitaryAssignments.tsx
│   │   ├── hooks.ts (useMilitaryAssignments)
│   │   ├── service.ts (MilitaryAssignmentsSnapshot, MilitaryUnitData, fetchMilitaryAssignments)
│   │   └── index.ts
│   ├── UnitOrders/
│   │   ├── UnitOrders.tsx
│   │   ├── hooks.ts (useUnitOrders)
│   │   ├── service.ts (UnitOrdersSnapshot, UnitOrder, fetchUnitOrders)
│   │   └── index.ts
│   ├── WorkerAssignments/
│   │   ├── WorkerAssignments.tsx
│   │   ├── hooks.ts (useWorkerStatus)
│   │   ├── service.ts (WorkerStatusSnapshot, WorkerAssignment, fetchWorkerStatus)
│   │   └── index.ts
│   └── ui/
│       └── Common.tsx (shared UI components)
├── contexts/
│   └── PollIntervalContext.tsx (global poll interval management)
├── App.tsx
└── main.tsx (wrapped with PollIntervalProvider)
```

## Key Changes

1. **Modular Structure**: Each component is self-contained with its own types, API calls, and hooks
2. **Global Poll Interval**: Managed via React Context (`PollIntervalContext`) instead of module-level variables
3. **Type Safety**: Types are colocated with the components that use them
4. **Clean Imports**: All components export through index.ts for cleaner imports
5. **No Shared API Folder**: Removed centralized api/ folder in favor of component-specific services

## Usage

Import components directly from their folders:
```typescript
import { BuildOrder } from './components/BuildOrder';
import { GameSpeed } from './components/GameSpeed';
```

Access poll interval from context:
```typescript
import { usePollInterval } from './contexts/PollIntervalContext';

const { pollInterval, setPollInterval } = usePollInterval();
```
