import resolve from '@rollup/plugin-node-resolve';
import commonjs from '@rollup/plugin-commonjs';
import { terser } from 'rollup-plugin-terser';

export default {
  input: 'node_modules/mediasoup-client/lib/index.js', // main entry of mediasoup-client
  output: {
    file: 'assets/mediasoup-client.bundle.js',
    format: 'umd',       // UMD for browser usage
    name: 'mediasoupClient' // global variable name
  },
  plugins: [
    resolve({ browser: true }),
    commonjs(),
    terser() // optional: minify
  ]
};
