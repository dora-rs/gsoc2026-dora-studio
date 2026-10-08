import { schemaSignatureFor, updatePortType } from './dataflow-schema'
import type { DataflowGraph } from './components/DataflowCanvas.vue'

const graph: DataflowGraph = {
  nodes: [
    {
      id: 'camera', operatorId: 'camera.py', runtime: 'python', position: { x: 0, y: 0 },
      inputs: {}, outputs: { frame: {} },
    },
    {
      id: 'detector', operatorId: 'detector.py', runtime: 'python', position: { x: 320, y: 0 },
      inputs: { frame: { source: 'camera/frame' } }, outputs: {},
    },
  ],
  edges: [{ id: 'camera-frame-detector-frame', sourceNode: 'camera', sourcePort: 'frame', targetNode: 'detector', targetPort: 'frame' }],
}

const initial = schemaSignatureFor(graph, [])
const withSourceType = updatePortType(graph, 'camera', 'frame', false, 'std/media/v1/Image')
if (withSourceType === graph) throw new Error('setting a known output type must produce a new graph')
if (graph.nodes[0].outputs.frame.type !== undefined) throw new Error('port updates must not mutate the original graph')
if (schemaSignatureFor(withSourceType, []) === initial) throw new Error('a source URN must change the schema signature')

const withBothTypes = updatePortType(withSourceType, 'detector', 'frame', true, 'std/control/v1/Twist')
const detector = withBothTypes.nodes.find(node => node.id === 'detector')!
if (detector.inputs.frame.type !== 'std/control/v1/Twist') throw new Error('sink URN was not assigned')
if (detector.inputs.frame.source !== 'camera/frame') throw new Error('input source must survive a type edit')
if (schemaSignatureFor(withBothTypes, []) === schemaSignatureFor(withSourceType, [])) throw new Error('a sink URN must change the schema signature')

const cleared = updatePortType(withBothTypes, 'detector', 'frame', true, '')
const clearedDetector = cleared.nodes.find(node => node.id === 'detector')!
if (clearedDetector.inputs.frame.type !== undefined) throw new Error('clearing a URN must remove only the type')
if (clearedDetector.inputs.frame.source !== 'camera/frame') throw new Error('clearing a URN must preserve the input source')

console.log('dataflow-schema tests passed')
