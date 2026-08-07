import assert from 'node:assert/strict';

import {
  NANO_ROBOT_BASE_STEP_METERS,
  NANO_ROBOT_BASE_TURN_STEP_RADIANS,
  applyNanoRobotBaseCommand,
  createNanoRobotBasePose,
  formatNanoRobotPoseValue,
  formatNanoRobotYawDegrees,
  normalizeNanoRobotYaw,
} from './nanoRobotMotion';

const EPSILON = 1e-10;

type TestCase = {
  name: string;
  run: () => void;
};

const assertClose = (actual: number, expected: number) => {
  assert.ok(
    Math.abs(actual - expected) < EPSILON,
    `expected ${actual} to be within ${EPSILON} of ${expected}`,
  );
};

const tests: TestCase[] = [
  {
    name: 'createNanoRobotBasePose returns the origin pose',
    run: () => {
      assert.deepEqual(createNanoRobotBasePose(), { x: 0, y: 0, yaw: 0 });
    },
  },
  {
    name: 'applyNanoRobotBaseCommand moves forward along yaw 0',
    run: () => {
      const pose = applyNanoRobotBaseCommand(createNanoRobotBasePose(), 'forward');

      assertClose(pose.x, NANO_ROBOT_BASE_STEP_METERS);
      assertClose(pose.y, 0);
      assertClose(pose.yaw, 0);
    },
  },
  {
    name: 'applyNanoRobotBaseCommand moves backward along yaw 0',
    run: () => {
      const pose = applyNanoRobotBaseCommand(createNanoRobotBasePose(), 'backward');

      assertClose(pose.x, -NANO_ROBOT_BASE_STEP_METERS);
      assertClose(pose.y, 0);
      assertClose(pose.yaw, 0);
    },
  },
  {
    name: 'applyNanoRobotBaseCommand turns left by 15 degrees',
    run: () => {
      const pose = applyNanoRobotBaseCommand(createNanoRobotBasePose(), 'turn-left');

      assertClose(pose.yaw, NANO_ROBOT_BASE_TURN_STEP_RADIANS);
      assert.equal(formatNanoRobotYawDegrees(pose.yaw), '15°');
    },
  },
  {
    name: 'applyNanoRobotBaseCommand resets to the origin pose',
    run: () => {
      const pose = applyNanoRobotBaseCommand({ x: 1.25, y: -0.5, yaw: Math.PI / 2 }, 'reset');

      assert.deepEqual(pose, { x: 0, y: 0, yaw: 0 });
    },
  },
  {
    name: 'applyNanoRobotBaseCommand moves forward along a 90 degree yaw',
    run: () => {
      const pose = applyNanoRobotBaseCommand({ x: 1, y: 2, yaw: Math.PI / 2 }, 'forward');

      assertClose(pose.x, 1);
      assertClose(pose.y, 2 + NANO_ROBOT_BASE_STEP_METERS);
      assertClose(pose.yaw, Math.PI / 2);
    },
  },
  {
    name: 'normalizeNanoRobotYaw wraps yaw into [-pi, pi)',
    run: () => {
      assertClose(normalizeNanoRobotYaw(Math.PI), -Math.PI);
      assertClose(normalizeNanoRobotYaw(3 * Math.PI), -Math.PI);
      assertClose(normalizeNanoRobotYaw(-3 * Math.PI), -Math.PI);
    },
  },
  {
    name: 'normalizeNanoRobotYaw avoids negative zero',
    run: () => {
      const yaw = normalizeNanoRobotYaw(-0);

      assert.equal(yaw, 0);
      assert.equal(Object.is(yaw, -0), false);
    },
  },
  {
    name: 'formatNanoRobotPoseValue keeps two decimals',
    run: () => {
      assert.equal(formatNanoRobotPoseValue(0), '0.00');
      assert.equal(formatNanoRobotPoseValue(1.234), '1.23');
    },
  },
];

let failures = 0;

for (const test of tests) {
  try {
    test.run();
    console.log(`ok - ${test.name}`);
  } catch (error) {
    failures += 1;
    console.error(`not ok - ${test.name}`);
    console.error(error);
  }
}

if (failures > 0) {
  process.exitCode = 1;
}
