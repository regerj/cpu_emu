module.exports = grammar({
  name: 'basm',

  rules: {
    source_file: $ => repeat(
        choice($.instruction, $.label)
    ),

    label: $ => /\w+:/,

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
      'jmp',
    ),

    operand: $ => seq(
      optional('*'),
      choice(
        $.register,
        $.immediate,
        $.label_use,
      )
    ),

    register: $ => seq(
        '$',
        /\w+/,
    ),

    immediate: $ => /\d+/,

    label_use: $ => /[a-zA-Z_]\w*/,
    
    comment: $ => seq(
        '//',
        /.*/,
    )
  }
});
