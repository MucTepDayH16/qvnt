from manim import *
from scipy.spatial.transform import Rotation as R


config.background_color = "#353535"
config.video_dir = r".\output"
config.partial_movie_dir = r".\media\videos\qubits"
config.frame_rate = 30
config.pixel_height = 240
config.pixel_width = 240


Xaxis = [1., 0., 0.]
Yaxis = [0., 1., 0.]
Zaxis = [0., 0., 1.]


phi = 75*DEGREES
theta = 30*DEGREES


def qubit(gl, start_point, rotate_axis, rotate_angle, show_basis, start_name, end_name, end_coef = None):
    rot = ValueTracker(0)
    point = np.array([
        np.cos(start_point[0] * DEGREES)*np.sin(start_point[1] * DEGREES),
        np.sin(start_point[0] * DEGREES)*np.sin(start_point[1] * DEGREES),
        np.cos(start_point[1] * DEGREES)
    ])

    axes = ThreeDAxes(
        x_range=[-2., 2., .5], x_length=4.,
        y_range=[-1.5, 1.5, .5], y_length=3.,
        z_range=[-2., 2., .5], z_length=4.
    )

    sphere = Sphere(resolution=(32,32))
    sphere.set_fill(BLUE, opacity=0.1)
    pole_0 = Dot3D(point=[0., 0., 1.], radius=0.03)
    pole_1 = Dot3D(point=[0., 0., -1.], radius=0.03)
    vec = Arrow3D(
        start=[0., 0., 0.],
        end=point,
        thickness=0.01
    )
    vec.set_fill(WHITE)
    vec_ref = vec.copy()

    vec.add_updater(
        lambda x: x.become(vec_ref.copy()).rotate_about_origin(
            rot.get_value() * DEGREES,
            axis=rotate_axis
        )
    )

    z = np.array(rotate_axis)
    x = point - np.dot(point, z) * z
    r = np.linalg.norm(x)
    x /= r
    y = np.cross(z, x)
    rotvec = R.from_matrix([x, y, z]).as_rotvec()
    angle = -np.linalg.norm(rotvec)

    arc = Arc(
        r, 0., 0., stroke_color=YELLOW
    ).shift(
        [[0., 0., np.dot(point, z)]]
    ).rotate_about_origin(
        angle, axis=rotvec
    )
    arc.add_updater(
        lambda x: x.become(
            Arc(
                r, 0., rot.get_value() * DEGREES, stroke_color=YELLOW
            ).shift(
                [[0., 0., np.dot(point, z)]]
            ).rotate_about_origin(
                angle, axis=rotvec
            )
        )
    )

    orbit = Circle()
    orbit.set_color(WHITE)

    gl.renderer.camera.light_source.move_to(2*IN)
    gl.renderer.camera.set_zoom(5.0)
    gl.set_camera_orientation(phi=phi, theta=theta)
    gl.add(axes, sphere, orbit, pole_0, pole_1, vec, arc)
    
    scene = VMobject()

    state_psi_end = MathTex(end_name, font_size=108)
    
    state_psi_end.move_to(
        Dot3D(point * 1.25).rotate_about_origin(
            rotate_angle * DEGREES,
            axis=rotate_axis
        )
    )    
    gl.renderer.camera.add_fixed_orientation_mobjects(state_psi_end)
    
    #state_psi_end.add_updater(
    #    lambda x: x.move_to(
    #        Dot3D(point * 1.25).rotate_about_origin(
    #            rot.get_value() * DEGREES,
    #            axis=rotate_axis
    #        )
    #    )
    #)


    state_psi = MathTex(start_name, font_size=108).move_to(
        point * 1.25
    )
    state_psi.add_updater(
        lambda x: x.move_to(
            Dot3D(point * 1.25).rotate_about_origin(
                rot.get_value() * DEGREES,
                axis=rotate_axis
            )
        )
    )
    gl.add_fixed_orientation_mobjects(state_psi)
    #gl.add(state_psi)

    if show_basis:
        state_0 = MathTex(r"|0\rangle", font_size=108).move_to(pole_0.get_center() * 1.25)
        state_1 = MathTex(r"|1\rangle", font_size=108).move_to(pole_1.get_center() * 1.25)
        gl.add_fixed_orientation_mobjects(state_0, state_1)

    state_rot = rot.animate.increment_value(rotate_angle)

    if end_coef is not None:
        state_psi_coef = MathTex(end_coef, font_size=108)
        state_psi_coef.move_to(
            state_psi_end
        ).shift(
            (np.sin(theta) * RIGHT + np.cos(theta) * DOWN) * (state_psi_end.width + state_psi_coef.width) * .13
        )
        gl.renderer.camera.add_fixed_orientation_mobjects(state_psi_coef)
        state_anim = AnimationGroup(
            Transform(state_psi, state_psi_end),
            Create(state_psi_coef)
        )
    else:
        state_anim = Transform(state_psi, state_psi_end)
    
    gl.wait(.5)
    gl.play(state_rot, run_time=1.5)
    gl.play(state_anim, run_time=.5)
    gl.wait(.5)


class X0(ThreeDScene):
    def construct(self):
        qubit(self, [0, 0], Xaxis, 180, False, r"|0\rangle", r"|1\rangle")


class X1(ThreeDScene):
    def construct(self):
        qubit(self, [0, 180], Xaxis, 180, False, r"|1\rangle", r"|0\rangle")


class X(ThreeDScene):
    def construct(self):
        qubit(self, [45, 45], Xaxis, 180, True, r"|\psi\rangle", r"|\psi\rangle")


class Y0(ThreeDScene):
    def construct(self):
        qubit(self, [0, 0], Yaxis, 180, False, r"|0\rangle", r"|1\rangle", "i")


class Y1(ThreeDScene):
    def construct(self):
        qubit(self, [0, 180], Yaxis, 180, False, r"|1\rangle", r"|0\rangle", "-i")


class Y(ThreeDScene):
    def construct(self):
        qubit(self, [45, 45], Yaxis, 180, True, r"|\psi\rangle", r"|\psi\rangle")


class Z0(ThreeDScene):
    def construct(self):
        qubit(self, [0, 0], Zaxis, 180, False, r"|0\rangle", r"|0\rangle")


class Z1(ThreeDScene):
    def construct(self):
        qubit(self, [0, 180], Zaxis, 180, False, r"|1\rangle", r"|1\rangle", "-")


class Z(ThreeDScene):
    def construct(self):
        qubit(self, [45, 45], Zaxis, 180, True, r"|\psi\rangle", r"|\psi\rangle")

class H0(ThreeDScene):
    def construct(self):
        qubit(self, [0, 0], [1./np.sqrt(2.), 0., 1./np.sqrt(2.)], 180, False, r"|0\rangle", r"|+\rangle")

class H1(ThreeDScene):
    def construct(self):
        qubit(self, [0, 180], [1./np.sqrt(2.), 0., 1./np.sqrt(2.)], 180, False, r"|1\rangle", r"|-\rangle")

class H(ThreeDScene):
    def construct(self):
        qubit(self, [45, 45], [1./np.sqrt(2.), 0., 1./np.sqrt(2.)], 180, True, r"|\psi\rangle", r"|\psi\rangle")