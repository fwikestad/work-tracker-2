export interface Customer {
  id: string;
  name: string;
  code: string | null;
  color: string | null;
  createdAt: string;
  updatedAt: string;
  archivedAt: string | null;
}

export interface WorkOrder {
  id: string;
  customerId: string;
  customerName: string | null;
  customerColor: string | null;
  name: string;
  code: string | null;
  description: string | null;
  servicenowTaskId: string | null;
  status: 'active' | 'paused' | 'closed';
  createdAt: string;
  updatedAt: string;
  archivedAt: string | null;
  isFavorite: boolean;
}

export interface ActivityType {
  id: string;
  name: string;
  sortOrder: number;
  createdAt: string;
}

export interface Session {
  id: string;
  workOrderId: string;
  workOrderName: string | null;
  customerName: string | null;
  customerColor: string | null;
  startTime: string;
  endTime: string | null;
  durationSeconds: number | null;
  activityType: string | null;
  notes: string | null;
  createdAt: string;
  updatedAt: string;
}

export interface ActiveSession {
  sessionId: string;
  workOrderId: string;
  workOrderName: string;
  customerName: string;
  customerColor: string | null;
  startedAt: string;
  elapsedSeconds: number;
}

export interface OrphanSession {
  sessionId: string;
  workOrderName: string;
  customerName: string;
  startedAt: string;
}

export interface QuickAddParams {
  customerName?: string;
  customerId?: string;
  workOrderName: string;
  workOrderCode?: string;
}

export interface QuickAddResult {
  customer: Customer;
  workOrder: WorkOrder;
  session: Session;
}

export interface DailySummary {
  date: string;
  totalSeconds: number;
  entries: SummaryEntry[];
  sessions: Session[];
}

export interface SummaryEntry {
  customerId: string;
  customerName: string;
  customerColor: string | null;
  workOrderId: string;
  workOrderName: string;
  totalSeconds: number;
  sessionCount: number;
}

export interface CreateCustomerParams {
  name: string;
  code?: string;
  color?: string;
}

export interface UpdateCustomerParams {
  name?: string;
  code?: string;
  color?: string;
}

export interface CreateWorkOrderParams {
  customerId: string;
  name: string;
  code?: string;
  description?: string;
  servicenowTaskId?: string | null;
}

export interface UpdateWorkOrderParams {
  name?: string;
  code?: string;
  description?: string;
  status?: 'active' | 'paused' | 'closed';
  servicenowTaskId?: string | null;
}

export interface UpdateSessionParams {
  startTime?: string;
  endTime?: string;
  activityType?: string;
  notes?: string;
}

export interface ReportData {
  startDate: string;
  endDate: string;
  totalSeconds: number;
  entries: ReportEntry[];
  sessions: Session[];
}

export interface ReportEntry {
  customerId: string;
  customerName: string;
  customerColor: string | null;
  workOrderId: string;
  workOrderName: string;
  totalSeconds: number;
  sessionCount: number;
}
