module.exports = grammar({
  name: 'basm',

  rules: {
    source_file: $ => repeat($.instruction),

    instruction: $ => seq(
      $.mnemonic,
      optional($.operand),
      ',',
      optional($.operand),
    ),

    mnemonic: $ => choice(
      'mov',
      'add',
      'sub',
    ),

    operand: $ => seq(
      optional('*'),
      choice(
        $.register,
        $.immediate,
      )
    ),

    register: $ => seq(
        '$',
        /\w+/,
    ),

    immediate: $ => /\d+/,
  }
});
