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

    testRepositoryInit() {
        const repo = Repository.init(this._tmpDirName);
        __self.assertFalse(repo.isBare());
        __self.assertTrue(repo.isEmpty());
    }

    testRepositoryInitBare() {
        const repo = Repository.init(this._tmpDirName, true);
        __self.assertTrue(repo.isBare());
        __self.assertTrue(repo.isEmpty());
        __self.assertNull(repo.namespace());
    }

    testRepositoryOpen() {
        Repository.init(this._tmpDirName);

        const repo = Repository.open(this._tmpDirName);
        __self.assertFalse(repo.isBare());
        __self.assertTrue(repo.isEmpty());
        __self.assertEquals(RepositoryState.Clean, repo.state());
    }

    testRepositoryOpenBare() {
        Repository.init(this._tmpDirName, true);

        const repo = Repository.open(this._tmpDirName);
        __self.assertTrue(repo.isBare());
        __self.assertTrue(repo.isEmpty());
        __self.assertEquals(RepositoryState.Clean, repo.state());
    }
}
