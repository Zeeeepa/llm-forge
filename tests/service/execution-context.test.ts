/**
 * Execution Context Tests
 *
 * Verifies the Agentics Foundational Execution Unit instrumentation:
 * - Span lifecycle (repo + agent)
 * - Header extraction and validation
 * - Artifact creation and attachment
 * - Invariant enforcement
 * - Response envelope shape
 * - JSON serializability
 *
 * @module tests/service/execution-context
 */

import { describe, it, expect } from 'vitest';
import {
  ExecutionContext,
  extractExecutionHeaders,
  createArtifactRef,
  type RepoSpan,
  type AgentSpan,
  type ArtifactRef,
} from '../../src/service/execution-context.js';

// =============================================================================
// EXECUTION CONTEXT
// =============================================================================

describe('ExecutionContext', () => {
  const EXECUTION_ID = '11111111-1111-1111-1111-111111111111';
  const PARENT_SPAN_ID = '22222222-2222-2222-2222-222222222222';

  describe('constructor', () => {
    it('creates a repo span on instantiation', () => {
      const ctx = new ExecutionContext(EXECUTION_ID, PARENT_SPAN_ID);

      expect(ctx.executionId).toBe(EXECUTION_ID);
      expect(ctx.repoSpan.type).toBe('repo');
      expect(ctx.repoSpan.repo_name).toBe('llm-forge');
      expect(ctx.repoSpan.parent_span_id).toBe(PARENT_SPAN_ID);
      expect(ctx.repoSpan.execution_id).toBe(EXECUTION_ID);
      expect(ctx.repoSpan.status).toBe('RUNNING');
      expect(ctx.repoSpan.span_id).toBeTruthy();
      expect(ctx.repoSpan.start_time).toBeTruthy();
      expect(ctx.repoSpan.agent_spans).toEqual([]);
    });

    it('generates a unique span_id for the repo span', () => {
      const ctx1 = new ExecutionContext(EXECUTION_ID, PARENT_SPAN_ID);
      const ctx2 = new ExecutionContext(EXECUTION_ID, PARENT_SPAN_ID);
      expect(ctx1.repoSpan.span_id).not.toBe(ctx2.repoSpan.span_id);
    });
  });

  describe('startAgentSpan', () => {
    it('creates an agent span nested under the repo span', () => {
      const ctx = new ExecutionContext(EXECUTION_ID, PARENT_SPAN_ID);
      const span = ctx.startAgentSpan('test-agent');

      expect(span.type).toBe('agent');
      expect(span.agent_name).toBe('test-agent');
      expect(span.repo_name).toBe('llm-forge');
      expect(span.parent_span_id).toBe(ctx.repoSpan.span_id);
      expect(span.status).toBe('RUNNING');
      expect(span.artifacts).toEqual([]);
      expect(span.span_id).toBeTruthy();
      expect(span.start_time).toBeTruthy();
    });

    it('appends the agent span to repo agent_spans', () => {
      const ctx = new ExecutionContext(EXECUTION_ID, PARENT_SPAN_ID);
      const span = ctx.startAgentSpan('agent-1');

      expect(ctx.repoSpan.agent_spans).toHaveLength(1);
      expect(ctx.repoSpan.agent_spans[0]).toBe(span);
    });

    it('supports multiple agent spans', () => {
      const ctx = new ExecutionContext(EXECUTION_ID, PARENT_SPAN_ID);
      const span1 = ctx.startAgentSpan('agent-1');
      const span2 = ctx.startAgentSpan('agent-2');

      expect(ctx.repoSpan.agent_spans).toHaveLength(2);
      expect(span1.span_id).not.toBe(span2.span_id);
    });

    it('attaches optional metadata', () => {
      const ctx = new ExecutionContext(EXECUTION_ID, PARENT_SPAN_ID);
      const span = ctx.startAgentSpan('test-agent', { version: '1.0.0', classification: 'GENERATION' });

      expect(span.metadata).toEqual({ version: '1.0.0', classification: 'GENERATION' });
    });
  });

  describe('completeAgentSpan', () => {
    it('marks the span as COMPLETED with end_time', () => {
      const ctx = new ExecutionContext(EXECUTION_ID, PARENT_SPAN_ID);
      const span = ctx.startAgentSpan('test-agent');
      ctx.completeAgentSpan(span);

      expect(span.status).toBe('COMPLETED');
      expect(span.end_time).toBeTruthy();
      expect(span.failure_reason).toBeUndefined();
    });
  });

  describe('failAgentSpan', () => {
    it('marks the span as FAILED with reason and end_time', () => {
      const ctx = new ExecutionContext(EXECUTION_ID, PARENT_SPAN_ID);
      const span = ctx.startAgentSpan('test-agent');
      ctx.failAgentSpan(span, 'Something went wrong');

      expect(span.status).toBe('FAILED');
      expect(span.end_time).toBeTruthy();
      expect(span.failure_reason).toBe('Something went wrong');
    });
  });

  describe('attachArtifact', () => {
    it('adds an artifact to the agent span', () => {
      const ctx = new ExecutionContext(EXECUTION_ID, PARENT_SPAN_ID);
      const span = ctx.startAgentSpan('test-agent');
      const artifact = createArtifactRef('test-artifact', '{"data": true}');
      ctx.attachArtifact(span, artifact);

      expect(span.artifacts).toHaveLength(1);
      expect(span.artifacts[0].label).toBe('test-artifact');
    });

    it('supports multiple artifacts on one span', () => {
      const ctx = new ExecutionContext(EXECUTION_ID, PARENT_SPAN_ID);
      const span = ctx.startAgentSpan('test-agent');
      ctx.attachArtifact(span, createArtifactRef('artifact-1', '{"a":1}'));
      ctx.attachArtifact(span, createArtifactRef('artifact-2', '{"b":2}'));

      expect(span.artifacts).toHaveLength(2);
    });
  });

  describe('finalize', () => {
    it('sets repo span to COMPLETED when all agent spans are COMPLETED', () => {
      const ctx = new ExecutionContext(EXECUTION_ID, PARENT_SPAN_ID);
      const span = ctx.startAgentSpan('test-agent');
      ctx.completeAgentSpan(span);

      const repo = ctx.finalize();

      expect(repo.status).toBe('COMPLETED');
      expect(repo.end_time).toBeTruthy();
      expect(repo.failure_reason).toBeUndefined();
    });

    it('sets repo span to FAILED when any agent span is FAILED', () => {
      const ctx = new ExecutionContext(EXECUTION_ID, PARENT_SPAN_ID);
      const span1 = ctx.startAgentSpan('agent-1');
      const span2 = ctx.startAgentSpan('agent-2');
      ctx.completeAgentSpan(span1);
      ctx.failAgentSpan(span2, 'Broke');

      const repo = ctx.finalize();

      expect(repo.status).toBe('FAILED');
      expect(repo.failure_reason).toContain('agent-2: Broke');
    });

    it('INVARIANT: returns FAILED when no agent spans exist', () => {
      const ctx = new ExecutionContext(EXECUTION_ID, PARENT_SPAN_ID);
      const repo = ctx.finalize();

      expect(repo.status).toBe('FAILED');
      expect(repo.failure_reason).toBe('No agent spans produced');
    });

    it('still includes all agent spans even on failure', () => {
      const ctx = new ExecutionContext(EXECUTION_ID, PARENT_SPAN_ID);
      const span1 = ctx.startAgentSpan('agent-1');
      const span2 = ctx.startAgentSpan('agent-2');
      ctx.failAgentSpan(span1, 'Error 1');
      ctx.failAgentSpan(span2, 'Error 2');

      const repo = ctx.finalize();

      expect(repo.agent_spans).toHaveLength(2);
      expect(repo.failure_reason).toContain('agent-1: Error 1');
      expect(repo.failure_reason).toContain('agent-2: Error 2');
    });

    it('is idempotent (calling finalize twice returns same structure)', () => {
      const ctx = new ExecutionContext(EXECUTION_ID, PARENT_SPAN_ID);
      const span = ctx.startAgentSpan('test-agent');
      ctx.completeAgentSpan(span);

      const repo1 = ctx.finalize();
      const repo2 = ctx.finalize();

      expect(repo1.status).toBe(repo2.status);
      expect(repo1.agent_spans).toBe(repo2.agent_spans);
    });
  });

  describe('buildResponse', () => {
    it('wraps result with execution envelope', () => {
      const ctx = new ExecutionContext(EXECUTION_ID, PARENT_SPAN_ID);
      const span = ctx.startAgentSpan('test-agent');
      ctx.completeAgentSpan(span);

      const response = ctx.buildResponse({ success: true, data: 42 });

      expect(response.result).toEqual({ success: true, data: 42 });
      expect(response.execution.type).toBe('repo');
      expect(response.execution.status).toBe('COMPLETED');
      expect(response.execution.agent_spans).toHaveLength(1);
    });

    it('preserves causal ordering via parent_span_id chain', () => {
      const ctx = new ExecutionContext(EXECUTION_ID, PARENT_SPAN_ID);
      const span = ctx.startAgentSpan('test-agent');
      ctx.completeAgentSpan(span);

      const response = ctx.buildResponse({});

      // Core -> Repo -> Agent chain
      expect(response.execution.parent_span_id).toBe(PARENT_SPAN_ID);
      expect(response.execution.agent_spans[0].parent_span_id).toBe(response.execution.span_id);
    });

    it('is JSON-serializable without loss', () => {
      const ctx = new ExecutionContext(EXECUTION_ID, PARENT_SPAN_ID);
      const span = ctx.startAgentSpan('test-agent', { version: '1.0.0' });
      ctx.attachArtifact(span, createArtifactRef('output', '{"result":"ok"}'));
      ctx.completeAgentSpan(span);

      const response = ctx.buildResponse({ success: true });
      const serialized = JSON.stringify(response);
      const deserialized = JSON.parse(serialized);

      expect(deserialized.result).toEqual({ success: true });
      expect(deserialized.execution.type).toBe('repo');
      expect(deserialized.execution.agent_spans[0].type).toBe('agent');
      expect(deserialized.execution.agent_spans[0].artifacts[0].label).toBe('output');
    });
  });
});

// =============================================================================
// HEADER EXTRACTION
// =============================================================================

describe('extractExecutionHeaders', () => {
  it('extracts execution_id and parent_span_id from headers', () => {
    const result = extractExecutionHeaders({
      'x-execution-id': 'exec-123',
      'x-parent-span-id': 'span-456',
    });

    expect(result).not.toBeNull();
    expect(result!.executionId).toBe('exec-123');
    expect(result!.parentSpanId).toBe('span-456');
  });

  it('returns null when parent_span_id is missing', () => {
    const result = extractExecutionHeaders({
      'x-execution-id': 'exec-123',
    });

    expect(result).toBeNull();
  });

  it('generates execution_id if only parent_span_id is present', () => {
    const result = extractExecutionHeaders({
      'x-parent-span-id': 'span-456',
    });

    expect(result).not.toBeNull();
    expect(result!.parentSpanId).toBe('span-456');
    expect(result!.executionId).toBeTruthy();
  });

  it('returns null when headers are empty', () => {
    const result = extractExecutionHeaders({});
    expect(result).toBeNull();
  });

  it('handles array header values (picks first)', () => {
    const result = extractExecutionHeaders({
      'x-execution-id': ['exec-1', 'exec-2'],
      'x-parent-span-id': ['span-1', 'span-2'],
    });

    expect(result).not.toBeNull();
    expect(result!.executionId).toBe('exec-1');
    expect(result!.parentSpanId).toBe('span-1');
  });
});

// =============================================================================
// ARTIFACT CREATION
// =============================================================================

describe('createArtifactRef', () => {
  it('creates an artifact with SHA-256 hash', () => {
    const artifact = createArtifactRef('test', '{"hello":"world"}');

    expect(artifact.artifact_id).toBeTruthy();
    expect(artifact.label).toBe('test');
    expect(artifact.content_hash).toMatch(/^[0-9a-f]{64}$/);
    expect(artifact.content_type).toBe('application/json');
    expect(artifact.size_bytes).toBe(Buffer.byteLength('{"hello":"world"}', 'utf-8'));
    expect(artifact.created_at).toBeTruthy();
  });

  it('uses custom content type when provided', () => {
    const artifact = createArtifactRef('test', 'data', 'text/plain');
    expect(artifact.content_type).toBe('text/plain');
  });

  it('produces deterministic hashes for same content', () => {
    const a1 = createArtifactRef('a', 'same-content');
    const a2 = createArtifactRef('b', 'same-content');
    expect(a1.content_hash).toBe(a2.content_hash);
  });

  it('produces different hashes for different content', () => {
    const a1 = createArtifactRef('a', 'content-1');
    const a2 = createArtifactRef('a', 'content-2');
    expect(a1.content_hash).not.toBe(a2.content_hash);
  });

  it('generates unique artifact_ids', () => {
    const a1 = createArtifactRef('a', 'content');
    const a2 = createArtifactRef('a', 'content');
    expect(a1.artifact_id).not.toBe(a2.artifact_id);
  });
});

// =============================================================================
// FULL SPAN HIERARCHY INVARIANTS
// =============================================================================

describe('Span Hierarchy Invariants', () => {
  it('maintains Core -> Repo -> Agent hierarchy', () => {
    const coreSpanId = 'core-span-id';
    const ctx = new ExecutionContext('exec-1', coreSpanId);
    const agent = ctx.startAgentSpan('my-agent');
    ctx.completeAgentSpan(agent);
    const response = ctx.buildResponse({});

    // Repo's parent is Core
    expect(response.execution.parent_span_id).toBe(coreSpanId);
    // Agent's parent is Repo
    expect(response.execution.agent_spans[0].parent_span_id).toBe(response.execution.span_id);
  });

  it('execution is INVALID without agent spans', () => {
    const ctx = new ExecutionContext('exec-1', 'parent-1');
    const response = ctx.buildResponse({});

    expect(response.execution.status).toBe('FAILED');
    expect(response.execution.failure_reason).toBe('No agent spans produced');
  });

  it('all spans are append-only (no removal)', () => {
    const ctx = new ExecutionContext('exec-1', 'parent-1');
    ctx.startAgentSpan('agent-1');
    ctx.startAgentSpan('agent-2');
    ctx.startAgentSpan('agent-3');

    expect(ctx.repoSpan.agent_spans).toHaveLength(3);
    // There is no API to remove spans - append-only by design
  });

  it('failed execution still returns all spans', () => {
    const ctx = new ExecutionContext('exec-1', 'parent-1');
    const agent1 = ctx.startAgentSpan('agent-ok');
    const agent2 = ctx.startAgentSpan('agent-fail');

    ctx.completeAgentSpan(agent1);
    ctx.failAgentSpan(agent2, 'Timeout');

    const response = ctx.buildResponse({ partial: true });

    expect(response.execution.status).toBe('FAILED');
    expect(response.execution.agent_spans).toHaveLength(2);
    expect(response.execution.agent_spans[0].status).toBe('COMPLETED');
    expect(response.execution.agent_spans[1].status).toBe('FAILED');
    // Result is still included
    expect(response.result).toEqual({ partial: true });
  });

  it('all repo_name fields are llm-forge', () => {
    const ctx = new ExecutionContext('exec-1', 'parent-1');
    const span = ctx.startAgentSpan('test');
    ctx.completeAgentSpan(span);

    expect(ctx.repoSpan.repo_name).toBe('llm-forge');
    expect(span.repo_name).toBe('llm-forge');
  });
});
