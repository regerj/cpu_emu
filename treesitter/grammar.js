module.exports = grammar({
  name: 'basm',

  rules: {
    source_file: $ => repeat($.instruction),

    instruction: $ => choice(
      $.comment,
      seq(
        $.mnemonic,
        optional($.operand),
        ',',
        optional($.operand),
      ),
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
    comment: $ => seq(
        '//',
        /.*/,
    )
  }
});
