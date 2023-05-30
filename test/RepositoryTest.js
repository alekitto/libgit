import { Repository, RepositoryState } from '../index';
import { randomBytes } from 'node:crypto';
import { tmpdir } from 'node:os';

const Filesystem = Jymfony.Component.Filesystem.Filesystem;
const TestCase = Jymfony.Component.Testing.Framework.TestCase;

const fs = new Filesystem();

export default class RepositoryTest extends TestCase {
    _tmpDirName;

    async beforeEach() {
        this._tmpDirName = tmpdir() + '/' + randomBytes(5).toString('base64').replace(/[^0-9a-z]/i, '-');
        await fs.mkdir(this._tmpDirName);
    }

    async afterEach() {
        await fs.remove(this._tmpDirName);
    }

    async testRepositoryInit() {
        const repo = await Repository.init(this._tmpDirName);
        __self.assertFalse(repo.isBare());
        __self.assertTrue(repo.isEmpty());
    }

    async testRepositoryInitBare() {
        const repo = await Repository.init(this._tmpDirName, true);
        __self.assertTrue(repo.isBare());
        __self.assertTrue(repo.isEmpty());
        __self.assertNull(repo.namespace());
    }

    async testRepositoryOpen() {
        await Repository.init(this._tmpDirName);

        const repo = await Repository.open(this._tmpDirName);
        __self.assertFalse(repo.isBare());
        __self.assertTrue(repo.isEmpty());
        __self.assertEquals(RepositoryState.Clean, await repo.state());
    }

    async testRepositoryOpenBare() {
        await Repository.init(this._tmpDirName, true);

        const repo = await Repository.open(this._tmpDirName);
        __self.assertTrue(repo.isBare());
        __self.assertTrue(repo.isEmpty());
        __self.assertEquals(RepositoryState.Clean, await repo.state());
    }
}
