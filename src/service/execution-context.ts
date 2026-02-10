/**
 * Agentics Foundational Execution Unit - Execution Context
 *
 * Provides execution span management for integrating LLM-Forge into
 * a hierarchical ExecutionGraph produced by an Agentics Core.
 *
 * Span hierarchy:
 *   Core (external)
 *     └─ Repo (this module creates)
 *         └─ Agent (one or more, per handler invocation)
 *
 * Invariants:
 * - Every externally-invoked operation must receive execution_id + parent_span_id
 * - Repo span is created on entry
 * - Every agent execution produces an agent span
 * - Artifacts attach to agent spans only
 * - No agent spans = execution INVALID (FAILED)
 * - On failure, all emitted spans are still returned
 *
 * @module service/execution-context
 */

import { randomUUID, createHash } from 'crypto';

// =============================================================================
// TYPES
// =============================================================================

export type SpanStatus = 'RUNNING' | 'COMPLETED' | 'FAILED';

export interface ArtifactRef {
  artifact_id: string;
  label: string;
  content_hash: string;
  content_type: string;
  size_bytes: number;
  created_at: string;
}

export interface AgentSpan {
  type: 'agent';
  agent_name: string;
  repo_name: 'llm-forge';
  span_id: string;
  parent_span_id: string;
  start_time: string;
  end_time?: string;
  status: SpanStatus;
  failure_reason?: string;
  artifacts: ArtifactRef[];
  metadata?: Record<string, unknown>;
}

export interface RepoSpan {
  type: 'repo';
  repo_name: 'llm-forge';
  span_id: string;
  parent_span_id: string;
  execution_id: string;
  start_time: string;
  end_time?: string;
  status: SpanStatus;
  failure_reason?: string;
  agent_spans: AgentSpan[];
}

export interface ExecutionResponse<T = unknown> {
  result: T;
  execution: RepoSpan;
}

// =============================================================================
// EXECUTION CONTEXT
// =============================================================================

export class ExecutionContext {
  readonly executionId: string;
  readonly repoSpan: RepoSpan;

  constructor(executionId: string, parentSpanId: string) {
    this.executionId = executionId;
    this.repoSpan = {
      type: 'repo',
      repo_name: 'llm-forge',
      span_id: randomUUID(),
      parent_span_id: parentSpanId,
      execution_id: executionId,
      start_time: new Date().toISOString(),
      status: 'RUNNING',
      agent_spans: [],
    };
  }

  startAgentSpan(agentName: string, metadata?: Record<string, unknown>): AgentSpan {
    const span: AgentSpan = {
      type: 'agent',
      agent_name: agentName,
      repo_name: 'llm-forge',
      span_id: randomUUID(),
      parent_span_id: this.repoSpan.span_id,
      start_time: new Date().toISOString(),
      status: 'RUNNING',
      artifacts: [],
      metadata,
    };
    this.repoSpan.agent_spans.push(span);
    return span;
  }

  completeAgentSpan(span: AgentSpan): void {
    span.end_time = new Date().toISOString();
    span.status = 'COMPLETED';
  }

  failAgentSpan(span: AgentSpan, reason: string): void {
    span.end_time = new Date().toISOString();
    span.status = 'FAILED';
    span.failure_reason = reason;
  }

  attachArtifact(span: AgentSpan, artifact: ArtifactRef): void {
    span.artifacts.push(artifact);
  }

  finalize(): RepoSpan {
    this.repoSpan.end_time = new Date().toISOString();

    if (this.repoSpan.agent_spans.length === 0) {
      this.repoSpan.status = 'FAILED';
      this.repoSpan.failure_reason = 'No agent spans produced';
      return this.repoSpan;
    }

    const anyFailed = this.repoSpan.agent_spans.some(s => s.status === 'FAILED');
    this.repoSpan.status = anyFailed ? 'FAILED' : 'COMPLETED';

    if (anyFailed) {
      this.repoSpan.failure_reason = this.repoSpan.agent_spans
        .filter(s => s.status === 'FAILED')
        .map(s => `${s.agent_name}: ${s.failure_reason}`)
        .join('; ');
    }

    return this.repoSpan;
  }

  buildResponse<T>(result: T): ExecutionResponse<T> {
    const execution = this.finalize();
    return { result, execution };
  }
}

// =============================================================================
// HEADER EXTRACTION
// =============================================================================

export function extractExecutionHeaders(
  headers: Record<string, string | string[] | undefined>
): { executionId: string; parentSpanId: string } | null {
  const parentSpanId = getHeader(headers, 'x-parent-span-id');

  if (!parentSpanId) {
    return null;
  }

  const executionId = getHeader(headers, 'x-execution-id') || randomUUID();

  return { executionId, parentSpanId };
}

function getHeader(
  headers: Record<string, string | string[] | undefined>,
  name: string
): string | undefined {
  const value = headers[name];
  return Array.isArray(value) ? value[0] : value;
}

// =============================================================================
// ARTIFACT HELPERS
// =============================================================================

export function createArtifactRef(
  label: string,
  content: string,
  contentType: string = 'application/json'
): ArtifactRef {
  const buffer = Buffer.from(content, 'utf-8');
  return {
    artifact_id: randomUUID(),
    label,
    content_hash: createHash('sha256').update(buffer).digest('hex'),
    content_type: contentType,
    size_bytes: buffer.byteLength,
    created_at: new Date().toISOString(),
  };
}
