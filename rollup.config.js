import resolve from '@rollup/plugin-node-resolve';
import commonjs from '@rollup/plugin-commonjs';
import { terser } from 'rollup-plugin-terser';

export default {
  input: 'node_modules/mediasoup-client/lib/index.js',
  output: {
    file: 'assets/mediasoup-client.bundle.js',
    format: 'umd',
    name: 'mediasoupClient'
  },
  plugins: [
    resolve({ browser: true }),
    commonjs(),
    terser()
  ]
};
