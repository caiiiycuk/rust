use crate::{cell::{RefCell}, cmp, collections::HashMap, convert::TryInto, io::SeekFrom, mem};

static mut MEMFS: Option<HashMap<String, Vec<u8>>> = None;

fn get_fs() -> &'static mut HashMap<String, Vec<u8>> {
    unsafe { MEMFS.get_or_insert(HashMap::new()) }
}

pub fn create(path: &str) -> Result<Handle, String> {
    let fs = get_fs();
    fs.insert(path.to_owned(), Vec::new());
    Ok(Handle {
        path: path.to_owned(),
        write: true,
        data_ptr: unsafe { mem::transmute::<&mut Vec<u8>, usize>(fs.get_mut(path).unwrap()) },
        rindex_ptr: RefCell::new(0),
    })
}

pub fn open(path: &str) -> Result<Handle, String> {
    let fs = get_fs();
    match fs.get_mut(path) {
        Some(contents) => Ok(Handle {
            path: path.to_owned(),
            write: false,
            data_ptr: unsafe { mem::transmute::<&mut Vec<u8>, usize>(contents) },
            rindex_ptr: RefCell::new(0),
        }),
        None => Err(format!("path {} dost not exist", path).to_owned()),
    }
}

pub struct Handle {
    pub path: String,
    write: bool,
    data_ptr: usize,
    rindex_ptr: RefCell<usize>,
}

impl Handle {
    fn data(&self) -> &mut Vec<u8> {
        unsafe { mem::transmute::<usize, &mut Vec<u8>>(self.data_ptr) }
    }

    pub fn read(&self, buf: &mut [u8]) -> Result<usize, String> {
        if self.write {
            return Err(format!("{} is not readable", self.path))
        }

        let data = self.data();
        let mut rindex = self.rindex_ptr.borrow_mut();
        let to_read = cmp::min(data.len() - *rindex, buf.len());
        for i in 0..to_read {
            buf[i] = data[*rindex + i];
        }
        *rindex += to_read;
        Ok(to_read)
    }

    pub fn seek(&self, pos: SeekFrom) -> Result<u64, String> {
        if self.write {
            return Err(format!("{} is not readable", self.path))
        }

        let data = self.data();
        let mut rindex = self.rindex_ptr.borrow_mut();

        match pos {
            SeekFrom::Start(pos) => *rindex = pos.try_into().unwrap(),
            SeekFrom::End(_) => return Err(format!("Unable to read beyond the file, {}", self.path)),
            SeekFrom::Current(pos) => {
                let upos: usize = pos.try_into().unwrap();
                *rindex += upos;
            }
        }

        if *rindex >= data.len() {
            *rindex = data.len() - 1
        }

        Ok((*rindex).try_into().unwrap())
    }

    pub fn write(&self, buf: &[u8]) -> Result<usize, String> {
        if !self.write {
            return Err(format!("{} is not writable", self.path))
        }

        self.data().extend(buf);
        Ok(buf.len())
    }
}

#[cfg(test)]
mod tests {
    use crate::{create, open};
    use crate::io::SeekFrom;

    #[test]
    fn write_read() {
        let handle = create("/tmp/1").unwrap();
        handle.write(&[1,2,3]).unwrap();

        let buf: &mut [u8; 2] = &mut [0, 0];
        let handle = open("/tmp/1").unwrap();

        let count = handle.read(buf).unwrap();
        assert_eq!(count, 2);
        assert_eq!(buf, &[1, 2]);

        let count = handle.read(buf).unwrap();
        assert_eq!(count, 1);
        assert_eq!(buf, &[3, 2]);
    }

    fn double_write() {
        create("/tmp/1").unwrap().write(&[1,2,3]).unwrap();
        create("/tmp/2").unwrap().write(&[4,5]).unwrap();

        let buf: &mut [u8; 3] = &mut [0, 0, 0];

        let handle = open("/tmp/1").unwrap();
        let count = handle.read(buf).unwrap();
        assert_eq!(count, 3);
        assert_eq!(buf, &[1, 2, 3]);

        let handle = open("/tmp/2").unwrap();
        let count = handle.read(buf).unwrap();
        assert_eq!(count, 2);
        assert_eq!(buf, &[4, 5, 3]);
    }

    fn seek() {
        create("/tmp/1").unwrap().write(&[1,2,3]).unwrap();
        let handle = open("/tmp/1").unwrap();
        handle.seek(SeekFrom::Start(1));

        let buf: &mut [u8; 3] = &mut [0, 0, 0];
        let count = handle.read(buf).unwrap();
        assert_eq!(count, 2);
        assert_eq!(buf, &[2, 3, 0]);
    }
}
