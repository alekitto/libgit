process.env.APP_DEBUG = '1';

require('@jymfony/autoloader');

const [ ,, ...argv ] = process.argv;
if (0 === argv.length) {
    argv.push('test/**/*Test.js');
}

const Runner = Jymfony.Component.Testing.Framework.Runner;
new Runner().run(argv);
