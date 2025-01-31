export default {
    extends: ["@commitlint/config-conventional"],
    ignores: [(message) => /^build\(deps\): bump .*$/m.test(message)],
};
