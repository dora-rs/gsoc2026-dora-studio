export const NANO_ARM_JOINT_NAMES = [
  'joint1',
  'joint2',
  'joint3',
  'joint4',
  'joint5',
  'joint6',
] as const

export type NanoArmJointName = (typeof NANO_ARM_JOINT_NAMES)[number]
export type NanoArmJointState = Record<NanoArmJointName, number>
export type NanoArmJointAxis = [number, number, number]

export const NANO_ARM_JOINT_AXES: Record<NanoArmJointName, NanoArmJointAxis> = {
  joint1: [0, 0, 1],
  joint2: [1, 0, 0],
  joint3: [1, 0, 0],
  joint4: [1, 0, 0],
  joint5: [0, 0.4226, -0.9063],
  joint6: [0, -0.9063, -0.4226],
}

export const NANO_ARM_JOINT_LIMITS: Record<NanoArmJointName, { lower: number; upper: number }> = {
  joint1: { lower: -2.618, upper: 2.618 },
  joint2: { lower: -2.618, upper: 2.618 },
  joint3: { lower: -2.618, upper: 2.618 },
  joint4: { lower: -2.618, upper: 2.618 },
  joint5: { lower: -2.618, upper: 2.618 },
  joint6: { lower: -1.57, upper: 1.57 },
}

export const NANO_FULL_ARM_JOINT_NAMES: Record<NanoArmJointName, string> = {
  joint1: 'STS3215_03a-v1_Revolute-45',
  joint2: 'STS3215_03a-v1-1_Revolute-49',
  joint3: 'STS3215_03a-v1-2_Revolute-51',
  joint4: 'STS3215_03a-v1-3_Revolute-53',
  joint5: 'STS3215_03a_Wrist_Roll-v1_Revolute-55',
  joint6: 'STS3215_03a-v1-4_Revolute-57',
}

const NANO_FULL_ARM_CONTROL_NAMES = Object.fromEntries(
  Object.entries(NANO_FULL_ARM_JOINT_NAMES).map(([controlName, modelJointName]) => [modelJointName, controlName]),
) as Record<string, NanoArmJointName>

const LEGACY_SNAPSHOT_JOINT_NAMES: Record<NanoArmJointName, string> = {
  joint1: 'shoulder_pan',
  joint2: 'shoulder_lift',
  joint3: 'elbow_flex',
  joint4: 'wrist_flex',
  joint5: 'wrist_roll',
  joint6: 'gripper',
}

export function findNanoArmSnapshotJoint<T extends { name: string }>(
  joints: T[],
  jointName: NanoArmJointName,
): T | undefined {
  return joints.find(
    (joint) => joint.name === jointName || joint.name === LEGACY_SNAPSHOT_JOINT_NAMES[jointName],
  )
}

export function resolveNanoArmJointName(modelJointName: string): NanoArmJointName | undefined {
  if (NANO_ARM_JOINT_NAMES.includes(modelJointName as NanoArmJointName)) {
    return modelJointName as NanoArmJointName
  }

  return NANO_FULL_ARM_CONTROL_NAMES[modelJointName]
}

export function createNanoArmJointState(): NanoArmJointState {
  return {
    joint1: 0.22,
    joint2: -0.48,
    joint3: 0.86,
    joint4: -0.34,
    joint5: 0.18,
    joint6: 0.03,
  }
}

export function seedNanoArmJointStateFromSnapshot(
  joints: Array<{ name: string; value: number }>,
): NanoArmJointState {
  const next = createNanoArmJointState()

  for (const jointName of NANO_ARM_JOINT_NAMES) {
    const snapshotJoint = findNanoArmSnapshotJoint(joints, jointName)

    if (snapshotJoint) {
      next[jointName] = snapshotJoint.value
    }
  }

  return next
}

export function buildNanoArmModelResources(backendBaseUrl: string) {
  const baseUrl = backendBaseUrl.replace(/\/+$/, '')

  return {
    xmlUrl: `${baseUrl}/models/nano_models/models/nano_full.xml`,
    assetBaseUrl: `${baseUrl}/models/nano_models/models/nano_assets/`,
  }
}

export type NanoArmModelSpec = {
  meshFiles: Record<string, string>
  jointOrder: NanoArmJointName[]
}

function isNanoArmJointName(name: string): name is NanoArmJointName {
  return NANO_ARM_JOINT_NAMES.includes(name as NanoArmJointName)
}

export function parseNanoArmXml(xmlText: string): NanoArmModelSpec {
  const doc = new DOMParser().parseFromString(xmlText, 'application/xml')
  const meshFiles = Object.fromEntries(
    Array.from(doc.querySelectorAll('asset mesh'))
      .map((mesh) => {
        const name = mesh.getAttribute('name')
        const file = mesh.getAttribute('file')
        return name && file ? [name, file] : null
      })
      .filter((entry): entry is [string, string] => entry !== null),
  )
  const jointOrder = Array.from(doc.querySelectorAll('actuator position'))
    .map((actuator) => actuator.getAttribute('joint'))
    .map((jointName) => (jointName ? resolveNanoArmJointName(jointName) : undefined))
    .filter((name): name is NanoArmJointName => Boolean(name))

  return {
    meshFiles,
    jointOrder: jointOrder.length > 0 ? jointOrder : [...NANO_ARM_JOINT_NAMES],
  }
}
