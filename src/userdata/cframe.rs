use super::{enums::RotationOrder, Vector3};


#[derive(Clone, Copy, PartialEq, PartialOrd, Debug)]
pub struct CFrame {
    rot_matrix: [[f64; 3]; 3],
    pos: [f64; 3]
}

impl Default for CFrame {
    fn default() -> Self {
        CFrame::IDENTITY
    }
}

impl CFrame {
    pub const IDENTITY: CFrame = CFrame {
        rot_matrix: [
            [1f64, 0f64, 0f64],
            [0f64, 1f64, 0f64],
            [0f64, 0f64, 1f64]
        ],
        pos: [0f64, 0f64, 0f64]
    };
    pub const fn new() -> Self {
        CFrame::IDENTITY
    }
    pub const fn new_with_position(pos: Vector3) -> Self {
        CFrame {
            rot_matrix: CFrame::IDENTITY.rot_matrix,
            pos: [pos.x, pos.y, pos.z]
        }
    }
    pub fn new_looking_at(pos: Vector3, lookAt: Vector3) -> Self {
        Self::look_at(pos, lookAt, None)
    }
    pub const fn new_quaternion(pos: Vector3, q_x: f64, q_y: f64, q_z: f64, q_w: f64) -> Self {
        let d = q_x*q_x+q_y*q_y+q_z*q_z+q_w*q_w;
        let s = 2.0/d;
        let (xs, ys, zs) = (q_x* s, q_y* s, q_z* s);
        let (wx, wy, wz) = (q_w*xs, q_w*ys, q_w*zs);
        let (xx, xy, xz) = (q_x*xs, q_x*ys, q_x*zs);
        let (yy, yz, zz) = (q_y*ys, q_y*zs, q_z*zs);
        Self {
            rot_matrix: [
                [1.0 - (yy+zz), xy - wz,         xz + wy        ],
                [xy + wz,       1.0 - (xx + zz), yz - wx        ],
                [xz - wy,       yz + wx,         1.0 - (xx + yy)]
            ],
            pos: [pos.x, pos.y, pos.z]
        }
    }
    pub const fn new_rot_matrix(pos: (f64, f64, f64), matrix: ((f64, f64, f64), (f64, f64, f64), (f64, f64, f64))) -> Self {
        CFrame {
            rot_matrix: [
                [matrix.0.0, matrix.0.1, matrix.0.2],
                [matrix.1.0, matrix.1.1, matrix.1.2],
                [matrix.2.0, matrix.2.1, matrix.2.2]
            ],
            pos: [pos.0, pos.1, pos.2]
        }
    }
    pub const fn from_matrix(pos: Vector3, r0: Vector3, r1: Vector3, r2: Vector3) -> Self {
        CFrame {
            rot_matrix: [
                [r0.x, r0.y, r0.z],
                [r1.x, r1.y, r1.z],
                [r2.x, r2.y, r2.z]
            ],
            pos: [pos.x, pos.y, pos.z]
        }
    }
    pub fn look_at(pos: Vector3, look_at: Vector3, up: Option<Vector3>) -> Self {
        Self::look_along(pos, look_at-pos, up)
    }
    pub fn look_along(pos: Vector3, direction: Vector3, up: Option<Vector3>) -> Self {
        let up = up.unwrap_or(Vector3::Y_AXIS);
        let v_z = direction.get_unit();
        let v_x = up.cross(v_z).get_unit();
        let v_y = v_z.cross(v_x);
        Self::from_matrix(pos, v_x, v_y, v_z)
    }
    pub fn from_rotation_between_vectors(from: Vector3, to: Vector3) -> Self {
        todo!()
    }
    pub fn from_euler_angles(rx: f64, ry: f64, rz: f64, order: Option<RotationOrder>) -> Self {
        todo!()
    }
    pub fn from_euler_angles_xyz(rx: f64, ry: f64, rz: f64) -> Self {
        Self::from_euler_angles(rx, ry, rz, Some(RotationOrder::XYZ))
    }
    pub fn from_euler_angles_yxz(rx: f64, ry: f64, rz: f64) -> Self {
        Self::from_euler_angles(rx, ry, rz, Some(RotationOrder::YXZ))
    }
    pub fn from_angles(rx: f64, ry: f64, rz: f64) -> Self {
        Self::from_euler_angles_xyz(rx, ry, rz)
    }
    pub fn from_orientation(rx: f64, ry: f64, rz: f64) -> Self {
        Self::from_euler_angles_yxz(rx, ry, rz)
    }
    //todo!()

}