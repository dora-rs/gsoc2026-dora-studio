import type { TypeRule } from './api'
import type { DataflowGraph } from './components/DataflowCanvas.vue'

/**
 * Every value that influences a connection's compatibility result.
 * Kept as a primitive string so Vue can reliably watch nested port updates.
 */
export function schemaSignatureFor(graph: DataflowGraph, typeRules: TypeRule[]): string {
  return JSON.stringify({
    edges: graph.edges.map(edge => [
      edge.sourceNode, edge.sourcePort, edge.targetNode, edge.targetPort,
    ]),
    ports: graph.nodes.map(node => ({
      id: node.id,
      inputs: Object.entries(node.inputs).map(([name, port]) => [name, port.type ?? null]),
      outputs: Object.entries(node.outputs).map(([name, port]) => [name, port.type ?? null]),
    })),
    typeRules: typeRules.map(rule => [rule.from, rule.to]),
  })
}

/**
 * Update one declared port type without mutating the input graph. Keeping the
 * original port fields matters for inputs: `source` must survive a type edit.
 */
export function updatePortType(
  graph: DataflowGraph,
  nodeId: string,
  portName: string,
  isInput: boolean,
  urn: string,
): DataflowGraph {
  let changed = false
  const nodes = graph.nodes.map(node => {
    if (node.id !== nodeId) return node
    const ports = isInput ? node.inputs : node.outputs
    const current = ports[portName]
    if (!current) return node

    const nextPort = { ...current }
    if (urn) nextPort.type = urn
    else delete nextPort.type
    changed = true

    return isInput
      ? { ...node, inputs: { ...node.inputs, [portName]: nextPort } }
      : { ...node, outputs: { ...node.outputs, [portName]: nextPort } }
  })
  return changed ? { ...graph, nodes } : graph
}
