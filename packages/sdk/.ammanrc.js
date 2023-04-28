
'use strict';
// @ts-check
const base = require('../../.ammanrc.js');
const validator = {
    ...base.validator,
    programs: [base.programs.token_auth_rules],
};
module.exports = {validator};
