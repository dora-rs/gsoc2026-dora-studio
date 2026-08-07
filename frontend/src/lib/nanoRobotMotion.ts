export type NanoRobotBasePose = {
  x: number;
  y: number;
  yaw: number;
};

export type NanoRobotBaseCommand = 'forward' | 'backward' | 'turn-left' | 'turn-right' | 'reset';

export const NANO_ROBOT_BASE_STEP_METERS = 0.08;
export const NANO_ROBOT_BASE_TURN_STEP_RADIANS = Math.PI / 12;

export const createNanoRobotBasePose = (): NanoRobotBasePose => ({ x: 0, y: 0, yaw: 0 });

export const normalizeNanoRobotYaw = (yaw: number): number => {
  const wrapped = ((((yaw + Math.PI) % (2 * Math.PI)) + 2 * Math.PI) % (2 * Math.PI)) - Math.PI;

  return Object.is(wrapped, -0) ? 0 : wrapped;
};

export const applyNanoRobotBaseCommand = (
  pose: NanoRobotBasePose,
  command: NanoRobotBaseCommand,
): NanoRobotBasePose => {
  if (command === 'reset') {
    return createNanoRobotBasePose();
  }

  if (command === 'turn-left') {
    return { ...pose, yaw: normalizeNanoRobotYaw(pose.yaw + NANO_ROBOT_BASE_TURN_STEP_RADIANS) };
  }

  if (command === 'turn-right') {
    return { ...pose, yaw: normalizeNanoRobotYaw(pose.yaw - NANO_ROBOT_BASE_TURN_STEP_RADIANS) };
  }

  const direction = command === 'forward' ? 1 : -1;
  const distance = direction * NANO_ROBOT_BASE_STEP_METERS;

  return {
    x: pose.x + Math.cos(pose.yaw) * distance,
    y: pose.y + Math.sin(pose.yaw) * distance,
    yaw: normalizeNanoRobotYaw(pose.yaw),
  };
};

export const formatNanoRobotPoseValue = (value: number): string => value.toFixed(2);

export const formatNanoRobotYawDegrees = (yaw: number): string =>
  `${Math.round((normalizeNanoRobotYaw(yaw) * 180) / Math.PI)}°`;
